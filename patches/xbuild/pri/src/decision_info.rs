use anyhow::{ensure, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::collections::hash_map::{Entry, HashMap};
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecisionInfo {
    qualifiers: Vec<Qualifier>,
    qualifier_sets: Vec<QualifierSet>,
    decisions: Vec<Decision>,
}

impl DecisionInfo {
    pub const IDENTIFIER: &'static [u8; 16] = b"[mrm_decn_info]\0";

    pub fn read<R: Read + Seek>(r: &mut R) -> Result<Self> {
        let num_distinct_qualifiers = r.read_u16::<LE>()? as usize;
        let num_qualifiers = r.read_u16::<LE>()? as usize;
        let num_qualifier_sets = r.read_u16::<LE>()? as usize;
        let num_decisions = r.read_u16::<LE>()? as usize;
        let num_index_table_entries = r.read_u16::<LE>()? as usize;
        let _total_data_length = r.read_u16::<LE>()?;
        let mut decision_infos = Vec::with_capacity(num_decisions);
        for _ in 0..num_decisions {
            let first_qualifier_set_index_index = r.read_u16::<LE>()? as usize;
            let num_qualifier_sets_in_decision = r.read_u16::<LE>()? as usize;
            decision_infos.push(DecisionInf {
                first_qualifier_set_index_index,
                num_qualifier_sets_in_decision,
            });
        }
        let mut qualifier_set_infos = Vec::with_capacity(num_qualifier_sets);
        for _ in 0..num_qualifier_sets {
            let first_qualifier_index_index = r.read_u16::<LE>()? as usize;
            let num_qualifiers_in_set = r.read_u16::<LE>()? as usize;
            qualifier_set_infos.push(QualifierSetInfo {
                first_qualifier_index_index,
                num_qualifiers_in_set,
            });
        }
        let mut qualifier_infos = Vec::with_capacity(num_qualifiers);
        for _ in 0..num_qualifiers {
            let index = r.read_u16::<LE>()? as usize;
            let priority = r.read_u16::<LE>()?;
            let fallback_score = r.read_u16::<LE>()?;
            ensure!(r.read_u16::<LE>()? == 0);
            qualifier_infos.push(QualifierInfo {
                index,
                priority,
                fallback_score,
            });
        }
        let mut distinct_qualifier_infos = Vec::with_capacity(num_distinct_qualifiers);
        for _ in 0..num_distinct_qualifiers {
            r.read_u16::<LE>()?;
            let qualifier_type = r.read_u16::<LE>()?;
            r.read_u16::<LE>()?;
            r.read_u16::<LE>()?;
            let operand_value_offset = r.read_u32::<LE>()?;
            distinct_qualifier_infos.push(DistinctQualifierInfo {
                qualifier_type,
                operand_value_offset,
            });
        }
        let mut index_table = Vec::with_capacity(num_index_table_entries);
        for _ in 0..num_index_table_entries {
            let index = r.read_u16::<LE>()?;
            index_table.push(index as usize);
        }
        let data_start = r.stream_position()?;
        let mut qualifiers = Vec::with_capacity(num_qualifiers);
        for info in &qualifier_infos {
            let distinct_info = &distinct_qualifier_infos[info.index];
            if let Some(qualifier_type) = QualifierType::from_u16(distinct_info.qualifier_type) {
                let string_start = data_start + distinct_info.operand_value_offset as u64 * 2;
                r.seek(SeekFrom::Start(string_start))?;
                let mut value = String::with_capacity(15);
                loop {
                    let c = r.read_u16::<LE>()?;
                    if c == 0 {
                        break;
                    }
                    value.push(char::from_u32(c as u32).unwrap());
                }
                qualifiers.push(Qualifier {
                    qualifier_type,
                    priority: info.priority,
                    fallback_score: info.fallback_score as f32 / 1000.0,
                    value,
                });
            }
        }
        let mut qualifier_sets = Vec::with_capacity(num_qualifier_sets);
        for info in &qualifier_set_infos {
            let mut qualifiers_in_set = Vec::with_capacity(info.num_qualifiers_in_set);
            for i in 0..info.num_qualifiers_in_set {
                qualifiers_in_set.push(index_table[info.first_qualifier_index_index + i]);
            }
            qualifier_sets.push(QualifierSet {
                qualifiers: qualifiers_in_set,
            });
        }
        let mut decisions = Vec::with_capacity(num_decisions);
        for info in &decision_infos {
            let mut qualifier_sets_in_decision =
                Vec::with_capacity(info.num_qualifier_sets_in_decision);
            for i in 0..info.num_qualifier_sets_in_decision {
                qualifier_sets_in_decision
                    .push(index_table[info.first_qualifier_set_index_index + i]);
            }
            decisions.push(Decision {
                qualifier_sets: qualifier_sets_in_decision,
            });
        }
        Ok(Self {
            qualifiers,
            qualifier_sets,
            decisions,
        })
    }

    pub fn write<W: Write + Seek>(&self, w: &mut W) -> Result<()> {
        let mut values = vec![];
        let mut distinct_qualifiers = HashMap::new();
        let mut distinct_qualifier_infos = Vec::with_capacity(self.num_qualifiers());
        let mut qualifier_infos = Vec::with_capacity(self.num_qualifiers());
        for qualifier in &self.qualifiers {
            let entry = distinct_qualifiers.entry((qualifier.qualifier_type, &qualifier.value));
            let index = distinct_qualifier_infos.len();
            if let Entry::Vacant(_) = entry {
                let current_offset = values.len() / 2;
                for c in qualifier.value.chars() {
                    values.write_u16::<LE>(c as u16)?;
                }
                values.write_u16::<LE>(0)?;
                distinct_qualifier_infos.push(DistinctQualifierInfo {
                    qualifier_type: qualifier.qualifier_type as u16,
                    operand_value_offset: current_offset as u32,
                });
            }
            let index = *entry.or_insert(index);
            qualifier_infos.push(QualifierInfo {
                index,
                priority: qualifier.priority,
                fallback_score: (qualifier.fallback_score * 1000.0) as u16,
            });
        }
        let mut qualifier_set_infos = Vec::with_capacity(self.num_qualifier_sets());
        let mut decision_infos = Vec::with_capacity(self.num_decisions());
        let mut index_table = Vec::with_capacity(self.num_qualifier_sets() + self.num_decisions());
        for qualifier_set in &self.qualifier_sets {
            let first_qualifier_index_index = index_table.len();
            let num_qualifiers_in_set = qualifier_set.qualifiers.len();
            for qualifier in &qualifier_set.qualifiers {
                index_table.push(*qualifier as u16);
            }
            qualifier_set_infos.push(QualifierSetInfo {
                first_qualifier_index_index,
                num_qualifiers_in_set,
            });
        }
        for decision in &self.decisions {
            let first_qualifier_set_index_index = index_table.len();
            let num_qualifier_sets_in_decision = decision.qualifier_sets.len();
            for qualifier_set in &decision.qualifier_sets {
                index_table.push(*qualifier_set as u16);
            }
            decision_infos.push(DecisionInf {
                first_qualifier_set_index_index,
                num_qualifier_sets_in_decision,
            });
        }
        w.write_u16::<LE>(distinct_qualifier_infos.len() as u16)?;
        w.write_u16::<LE>(qualifier_infos.len() as u16)?;
        w.write_u16::<LE>(qualifier_set_infos.len() as u16)?;
        w.write_u16::<LE>(decision_infos.len() as u16)?;
        w.write_u16::<LE>(index_table.len() as u16)?;
        w.write_u16::<LE>(0)?;
        let start = w.stream_position()?;
        for info in decision_infos {
            w.write_u16::<LE>(info.first_qualifier_set_index_index as u16)?;
            w.write_u16::<LE>(info.num_qualifier_sets_in_decision as u16)?;
        }
        for info in qualifier_set_infos {
            w.write_u16::<LE>(info.first_qualifier_index_index as u16)?;
            w.write_u16::<LE>(info.num_qualifiers_in_set as u16)?;
        }
        for info in qualifier_infos {
            w.write_u16::<LE>(info.index as u16)?;
            w.write_u16::<LE>(info.priority)?;
            w.write_u16::<LE>(info.fallback_score)?;
            w.write_u16::<LE>(0)?;
        }
        for info in distinct_qualifier_infos {
            w.write_u16::<LE>(0)?;
            w.write_u16::<LE>(info.qualifier_type)?;
            w.write_u16::<LE>(0)?;
            w.write_u16::<LE>(0)?;
            w.write_u32::<LE>(info.operand_value_offset)?;
        }
        for index in index_table {
            w.write_u16::<LE>(index)?;
        }
        w.write_all(&values)?;
        let end = w.stream_position()?;
        w.seek(SeekFrom::Start(start - 2))?;
        w.write_u16::<LE>((end - start) as u16)?;
        w.seek(SeekFrom::Start(end))?;
        Ok(())
    }

    pub fn num_qualifiers(&self) -> usize {
        self.qualifiers.len()
    }

    pub fn qualifier(&self, index: usize) -> Option<&Qualifier> {
        self.qualifiers.get(index)
    }

    pub fn add_qualifier(&mut self, qualifier: Qualifier) -> usize {
        let index = self.qualifiers.len();
        self.qualifiers.push(qualifier);
        index
    }

    pub fn num_qualifier_sets(&self) -> usize {
        self.qualifier_sets.len()
    }

    pub fn qualifier_set(&self, index: usize) -> Option<&QualifierSet> {
        self.qualifier_sets.get(index)
    }

    pub fn add_qualifier_set(&mut self, qualifier_set: QualifierSet) -> usize {
        let index = self.qualifier_sets.len();
        self.qualifier_sets.push(qualifier_set);
        index
    }

    pub fn num_decisions(&self) -> usize {
        self.decisions.len()
    }

    pub fn decision(&self, index: usize) -> Option<&Decision> {
        self.decisions.get(index)
    }

    pub fn add_decision(&mut self, decision: Decision) -> usize {
        let index = self.decisions.len();
        self.decisions.push(decision);
        index
    }
}

struct DecisionInf {
    first_qualifier_set_index_index: usize,
    num_qualifier_sets_in_decision: usize,
}

struct QualifierSetInfo {
    first_qualifier_index_index: usize,
    num_qualifiers_in_set: usize,
}

struct QualifierInfo {
    index: usize,
    priority: u16,
    fallback_score: u16,
}

struct DistinctQualifierInfo {
    qualifier_type: u16,
    operand_value_offset: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Qualifier {
    pub qualifier_type: QualifierType,
    pub priority: u16,
    pub fallback_score: f32,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifierSet {
    pub qualifiers: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    pub qualifier_sets: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u16)]
pub enum QualifierType {
    Language,
    Contrast,
    Scale,
    HomeRegion,
    TargetSize,
    LayoutDirection,
    Theme,
    AlternateForm,
    DXFeatureLevel,
    Configuration,
    DeviceFamily,
    Custom,
}

impl QualifierType {
    pub fn from_u16(qt: u16) -> Option<Self> {
        Some(match qt {
            0 => Self::Language,
            1 => Self::Contrast,
            2 => Self::Scale,
            3 => Self::HomeRegion,
            4 => Self::TargetSize,
            5 => Self::LayoutDirection,
            6 => Self::Theme,
            7 => Self::AlternateForm,
            8 => Self::DXFeatureLevel,
            9 => Self::Configuration,
            10 => Self::DeviceFamily,
            11 => Self::Custom,
            _ => return None,
        })
    }
}
