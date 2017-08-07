pub enum Inst<'a> {
    AInst { address: i16 },
    CInst {
        comp: &'a str,
        dest: Option<&'a str>,
        jump: Option<&'a str>,
    },
}
