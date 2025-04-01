use anyhow::{ensure, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::collections::BTreeSet;
use std::io::{Read, Write};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceMap {
    pub hierarchical_schema_section: u16,
    pub decision_info_section: u16,
    pub item_to_item_info_groups: Vec<ItemToItemInfoGroup>,
    pub item_info_groups: Vec<ItemInfoGroup>,
    pub item_infos: Vec<ItemInfo>,
    pub candidate_infos: Vec<CandidateInfo>,
}

impl ResourceMap {
    pub const IDENTIFIER: &'static [u8; 16] = b"[mrm_res_map2_]\0";

    pub fn read(r: &mut impl Read) -> Result<Self> {
        let environment_references_length = r.read_u16::<LE>()?;
        let num_environment_references = r.read_u16::<LE>()?;
        ensure!(environment_references_length == 0);
        ensure!(num_environment_references == 0);
        let hierarchical_schema_section = r.read_u16::<LE>()?;
        let _hierarchical_schema_reference_length = r.read_u16::<LE>()?;
        let decision_info_section = r.read_u16::<LE>()?;
        let resource_value_type_table_size = r.read_u16::<LE>()? as usize;
        let item_to_item_info_group_count = r.read_u16::<LE>()? as usize;
        let item_info_group_count = r.read_u16::<LE>()? as usize;
        let item_info_count = r.read_u32::<LE>()? as usize;
        let num_candidates = r.read_u32::<LE>()? as usize;
        let _data_length = r.read_u32::<LE>()?;
        let large_table_length = r.read_u32::<LE>()?;
        ensure!(large_table_length == 0);
        let mut resource_value_type_table = Vec::with_capacity(resource_value_type_table_size);
        for _ in 0..resource_value_type_table_size {
            ensure!(r.read_u32::<LE>()? == 4);
            let resource_value_type = r.read_u32::<LE>()?;
            resource_value_type_table.push(resource_value_type);
        }
        let mut item_to_item_info_groups = Vec::with_capacity(item_to_item_info_group_count);
        for _ in 0..item_to_item_info_group_count {
            let first_item = r.read_u16::<LE>()? as u32;
            let item_info_group = r.read_u16::<LE>()? as u32;
            item_to_item_info_groups.push(ItemToItemInfoGroup {
                first_item,
                item_info_group,
            });
        }
        let mut item_info_groups = Vec::with_capacity(item_info_group_count);
        for _ in 0..item_info_group_count {
            let group_size = r.read_u16::<LE>()? as u32;
            let first_item_info = r.read_u16::<LE>()? as u32;
            item_info_groups.push(ItemInfoGroup {
                group_size,
                first_item_info,
            });
        }
        let mut item_infos = Vec::with_capacity(item_info_count);
        for _ in 0..item_info_count {
            let decision = r.read_u16::<LE>()? as u32;
            let first_candidate = r.read_u16::<LE>()? as u32;
            item_infos.push(ItemInfo {
                decision,
                first_candidate,
            });
        }
        /*if large_table_length > 0 {
            let item_to_item_info_group_count_large = r.read_u32::<LE>()?;
            let item_info_group_count_large = r.read_u32::<LE>()?;
            let item_info_count_large = r.read_u32::<LE>()?;
            for _ in 0..item_to_item_info_group_count_large {
                let first_item = r.read_u32::<LE>()?;
                let item_info_group = r.read_u32::<LE>()?;
                item_to_item_info_groups.push(ItemToItemInfoGroup {
                    first_item,
                    item_info_group,
                });
            }
            for _ in 0..item_info_group_count_large {
                let group_size = r.read_u32::<LE>()?;
                let first_item_info = r.read_u32::<LE>()?;
                item_info_groups.push(ItemInfoGroup {
                    group_size,
                    first_item_info,
                });
            }
            for _ in 0..item_info_count_large {
                let decision = r.read_u32::<LE>()?;
                let first_candidate = r.read_u32::<LE>()?;
                item_infos.push(ItemInfo {
                    decision,
                    first_candidate,
                });
            }
        }*/
        let mut candidate_infos = Vec::with_capacity(num_candidates);
        for _ in 0..num_candidates {
            ensure!(r.read_u8()? == 0x01);
            let resource_value_type = resource_value_type_table[r.read_u8()? as usize];
            let source_file_index = r.read_u16::<LE>()?;
            let data_item_index = r.read_u16::<LE>()?;
            let data_item_section = r.read_u16::<LE>()?;
            candidate_infos.push(CandidateInfo {
                resource_value_type,
                source_file_index,
                data_item_index,
                data_item_section,
            });
            /*let candidate_sets = HashMap::new();
            for item_to_item_info_group in &item_to_item_info_groups {
                let item_info_group = if item_to_item_info_group.item_info_group < item_info_groups.len() {
                    item_info_groups[item_to_item_info_group.item_info_group]
                } else {
                    ItemInfoGroup {
                        group_size: 1,
                        first_item_info: item_to_item_info_group.item_info_group - item_info_groups.len(),
                    }
                };
                for i in 0..item_info_group.group_size {
                    let item_info = item_infos[item_info_group.first_item_info + i];
                    let decision =

                }
            }*/
        }
        Ok(Self {
            hierarchical_schema_section,
            decision_info_section,
            item_to_item_info_groups,
            item_info_groups,
            item_infos,
            candidate_infos,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
        let mut resource_value_type_table = BTreeSet::new();
        for candidate in &self.candidate_infos {
            resource_value_type_table.insert(candidate.resource_value_type);
        }
        w.write_u16::<LE>(0)?;
        w.write_u16::<LE>(0)?;
        w.write_u16::<LE>(self.hierarchical_schema_section)?;
        // TODO: hierarchical_schema_reference_length
        w.write_u16::<LE>(0)?;
        w.write_u16::<LE>(self.decision_info_section)?;
        w.write_u16::<LE>(resource_value_type_table.len() as _)?;
        w.write_u16::<LE>(self.item_to_item_info_groups.len() as _)?;
        w.write_u16::<LE>(self.item_info_groups.len() as _)?;
        w.write_u32::<LE>(self.item_infos.len() as _)?;
        w.write_u32::<LE>(self.candidate_infos.len() as _)?;
        // TODO: data_length
        w.write_u32::<LE>(0)?;
        w.write_u32::<LE>(0)?;
        for resource_value_type in &resource_value_type_table {
            w.write_u32::<LE>(4)?;
            w.write_u32::<LE>(*resource_value_type)?;
        }
        for item_to_item_info_group in &self.item_to_item_info_groups {
            w.write_u16::<LE>(item_to_item_info_group.first_item as u16)?;
            w.write_u16::<LE>(item_to_item_info_group.item_info_group as u16)?;
        }
        for item_info_group in &self.item_info_groups {
            w.write_u16::<LE>(item_info_group.group_size as u16)?;
            w.write_u16::<LE>(item_info_group.first_item_info as u16)?;
        }
        for item_info in &self.item_infos {
            w.write_u16::<LE>(item_info.decision as u16)?;
            w.write_u16::<LE>(item_info.first_candidate as u16)?;
        }
        for candidate in &self.candidate_infos {
            w.write_u8(0x01)?;
            let resource_value_type_index = resource_value_type_table
                .iter()
                .position(|t| *t == candidate.resource_value_type)
                .unwrap();
            w.write_u8(resource_value_type_index as u8)?;
            w.write_u16::<LE>(candidate.source_file_index)?;
            w.write_u16::<LE>(candidate.data_item_index)?;
            w.write_u16::<LE>(candidate.data_item_section)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ItemToItemInfoGroup {
    pub first_item: u32,
    pub item_info_group: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ItemInfoGroup {
    pub group_size: u32,
    pub first_item_info: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ItemInfo {
    pub decision: u32,
    pub first_candidate: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CandidateInfo {
    pub resource_value_type: u32,
    pub source_file_index: u16,
    pub data_item_index: u16,
    pub data_item_section: u16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResourceValueType {
    String,
    Path,
    EmbeddedData,
    AsciiString,
    Utf8String,
    AsciiPath,
    Utf8Path,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CandidateSet {
    pub resource_map_item: u32,
    pub decision_index: u16,
    pub candidates: Vec<Candidate>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Candidate {
    pub qualifier_set: u16,
    pub ty: ResourceValueType,
    pub data_item_section: u16,
    pub data_item_index: u16,
}
