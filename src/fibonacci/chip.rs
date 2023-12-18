use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, AssignedCell, Value},
    plonk::{ConstraintSystem, Error, Column, Advice, Instance, Selector},
    poly::Rotation,
};
use crate::config::FiboConfig;

pub struct FiboChip<F: Field> {
    config: FiboConfig,
    _marker: std::marker::PhantomData<F>,
}

impl<F: Field> FiboChip<F> {
    pub fn construct(config: FiboConfig) -> Self {
        Self {
            config,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advices: [Column<Advice>; 3], // [a, b, c]
        i: Column<Instance>,
        s: Selector
    ) -> FiboConfig {
        let [a, b, c] = advices;

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(i);

        meta.create_gate("add", |meta| {
            let aa = meta.query_advice(a, Rotation::cur());
            let bb = meta.query_advice(b, Rotation::cur());
            let cc = meta.query_advice(c, Rotation::cur());
            let s_add = meta.query_selector(s);
            vec![s_add * (aa + bb - cc)]
        });

        FiboConfig {
            a, b, c, i, s
        }
    }

    pub fn load_first_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: Value<F>,
        b: Value<F>,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                // enable selector for addition
                self.config.s.enable(&mut region, 0)?;

                // load f(0)
                let a_cell = region.assign_advice(
                    || "f(0) = 1",
                    self.config.a,
                    0,
                    || a,
                )?;

                // load f(1)
                let b_cell = region.assign_advice(
                    || "f(1) = 1",
                    self.config.b,
                    0,
                    || b,
                )?;

                // load f(2)
                let c_cell = region.assign_advice(
                    || "f(2)",
                    self.config.c,
                    0,
                    || a_cell.value().copied() + b_cell.value()
                )?;

                // return
                Ok((a_cell, b_cell, c_cell))
            },
        )
    }

    pub fn load_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_b: AssignedCell<F, F>,
        prev_c: AssignedCell<F, F>,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                // enable selector for addition
                self.config.s.enable(&mut region, 0)?;

                // load prev row
                let curr_a = prev_b.copy_advice(
                    || "prev_b to curr_a",
                    &mut region,
                    self.config.a,
                    0,
                )?;

                let curr_b = prev_c.copy_advice(
                    || "prev_c to curr_b",
                    &mut region,
                    self.config.b,
                    0,
                )?;

                let curr_c = region.assign_advice(
                    || "f(2)",
                    self.config.c,
                    0,
                    || curr_a.value().copied() + curr_b.value()
                )?;

                // return
                Ok((curr_a, curr_b, curr_c))
            },
        )
    }

    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.i, row)
    }
}
