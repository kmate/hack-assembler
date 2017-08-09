#[derive(Debug, PartialEq)]
pub enum Inst<'a> {
    AInst { address: u16 },
    CInst {
        comp: &'a str,
        dest: Option<&'a str>,
        jump: Option<&'a str>,
    },
}
