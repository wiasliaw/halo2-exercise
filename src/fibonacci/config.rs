use halo2_proofs::plonk::{Advice, Column, Instance, Selector};

#[derive(Clone, Debug)]
pub struct FiboConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub c: Column<Advice>,
    pub i: Column<Instance>,
    pub s: Selector,
}
