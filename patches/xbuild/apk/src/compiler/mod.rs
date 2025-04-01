use crate::manifest::AndroidManifest;
use crate::res::{
    Chunk, ResTableConfig, ResTableEntry, ResTableHeader, ResTablePackageHeader,
    ResTableTypeHeader, ResTableTypeSpecHeader, ResTableValue, ResValue, ScreenType,
};
use anyhow::Result;

mod attributes;
mod table;
mod xml;

pub use table::Table;

pub fn compile_manifest(manifest: &AndroidManifest, table: &Table) -> Result<Chunk> {
    let xml = quick_xml::se::to_string(manifest)?;
    xml::compile_xml(&xml, table)
}

const DPI_SIZE: [u32; 5] = [48, 72, 96, 144, 192];

fn variants(name: &str) -> impl Iterator<Item = (String, u32)> + '_ {
    DPI_SIZE
        .into_iter()
        .map(move |size| (format!("res/{0}/{0}{1}.png", name, size), size))
}

pub fn compile_mipmap<'a>(package_name: &str, name: &'a str) -> Result<Mipmap<'a>> {
    let chunk = Chunk::Table(
        ResTableHeader { package_count: 1 },
        vec![
            Chunk::StringPool(variants(name).map(|(res, _)| res).collect(), vec![]),
            Chunk::TablePackage(
                ResTablePackageHeader {
                    id: 127,
                    name: package_name.to_string(),
                    type_strings: 288,
                    last_public_type: 1,
                    key_strings: 332,
                    last_public_key: 1,
                    type_id_offset: 0,
                },
                vec![
                    Chunk::StringPool(vec!["mipmap".to_string()], vec![]),
                    Chunk::StringPool(vec!["icon".to_string()], vec![]),
                    Chunk::TableTypeSpec(
                        ResTableTypeSpecHeader {
                            id: 1,
                            res0: 0,
                            res1: 0,
                            entry_count: 1,
                        },
                        vec![256],
                    ),
                    mipmap_table_type(1, 160, 0),
                    mipmap_table_type(1, 240, 1),
                    mipmap_table_type(1, 320, 2),
                    mipmap_table_type(1, 480, 3),
                    mipmap_table_type(1, 640, 4),
                ],
            ),
        ],
    );
    Ok(Mipmap { name, chunk })
}

fn mipmap_table_type(type_id: u8, density: u16, string_id: u32) -> Chunk {
    Chunk::TableType(
        ResTableTypeHeader {
            id: type_id,
            res0: 0,
            res1: 0,
            entry_count: 1,
            entries_start: 88,
            config: ResTableConfig {
                size: 28 + 36,
                imsi: 0,
                locale: 0,
                screen_type: ScreenType {
                    orientation: 0,
                    touchscreen: 0,
                    density,
                },
                input: 0,
                screen_size: 0,
                version: 4,
                unknown: vec![0; 36],
            },
        },
        vec![0],
        vec![Some(ResTableEntry {
            size: 8,
            flags: 0,
            key: 0,
            value: ResTableValue::Simple(ResValue {
                size: 8,
                res0: 0,
                data_type: 3,
                data: string_id,
            }),
        })],
    )
}

pub struct Mipmap<'a> {
    name: &'a str,
    chunk: Chunk,
}

impl<'a> Mipmap<'a> {
    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn variants(&self) -> impl Iterator<Item = (String, u32)> + 'a {
        variants(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::table::Ref;
    use crate::manifest::Activity;
    use std::io::Cursor;

    #[test]
    fn test_compile_mipmap() -> Result<()> {
        crate::tests::init_logger();
        let mipmap = compile_mipmap("com.example.helloworld", "icon")?;
        let mut buf = vec![];
        let mut cursor = Cursor::new(&mut buf);
        mipmap.chunk().write(&mut cursor)?;
        let mut cursor = Cursor::new(&buf);
        let chunk = Chunk::parse(&mut cursor)?;
        println!("{:#?}", mipmap.chunk());
        println!("{:#?}", chunk);
        assert_eq!(*mipmap.chunk(), chunk);
        Ok(())
    }

    #[test]
    fn test_lookup_attr() -> Result<()> {
        let android = crate::tests::android_jar(31)?;
        let mut table = Table::default();
        table.import_apk(&android)?;
        let entry = table.entry_by_ref(Ref::attr("compileSdkVersionCodename"))?;
        assert_eq!(u32::from(entry.id()), 0x01010573);
        Ok(())
    }

    #[test]
    fn test_compile_manifest() -> Result<()> {
        let android = crate::tests::find_android_jar()?;
        let mut table = Table::default();
        table.import_apk(&android)?;
        let mut manifest = AndroidManifest::default();
        manifest.application.label = Some("helloworld".into());
        manifest.application.theme = Some("@android:style/Theme.Light.NoTitleBar".into());
        manifest.application.debuggable = Some(true);
        let activity = Activity {
            config_changes: Some("orientation|keyboardHidden".into()),
            launch_mode: Some("singleTop".into()),
            ..Default::default()
        };
        manifest.application.activities.push(activity);
        let _chunk = compile_manifest(&manifest, &table)?;
        Ok(())
    }
}
