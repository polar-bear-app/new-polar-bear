/*use anyhow::Result;
use std::io::Cursor;
use xmsix::pri::PriFile;

const PRI_BYTES: &'static [u8] = include_bytes!("resources.pri");

#[test]
fn test_read() -> Result<()> {
    let mut cursor = Cursor::new(PRI_BYTES);
    let pri = PriFile::read(&mut cursor)?;
    let mut pri2 = vec![];
    let mut cursor = Cursor::new(&mut pri2);
    pri.write(&mut cursor)?;
    let mut cursor = Cursor::new(&pri2);
    let _pri2 = PriFile::read(&mut cursor)?;
    //println!("{:#?}", pri);
    //assert!(false);
    Ok(())
}*/
