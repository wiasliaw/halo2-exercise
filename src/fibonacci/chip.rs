use halo2_proofs::{
    arithmetic::Field,
    circuit::{Chip, Layouter, AssignedCell, Value},
    plonk::{ConstraintSystem, Error, Column, Advice, Instance, Selector},
    poly::Rotation,
};
use crate::config::FiboConfig;

/**
 * instruction interface
 */

pub trait FiboInstructions<F: Field>: Chip<F> {
    fn write_first_row(
        &self,
        layouter: impl Layouter<F>,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error>;

    fn write_next_row(
        &self,
        layouter: impl Layouter<F>,
        prev_b: AssignedCell<F, F>,
        prev_c: AssignedCell<F, F>,
        i: usize,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error>;

    fn expose_public(
        &self,
        layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
    ) -> Result<(), Error>;
}

/**
 * Chip and implementation
 */

pub struct FiboChip<F: Field> {
    config: FiboConfig,
    _marker: std::marker::PhantomData<F>,
}

impl<F: Field> Chip<F> for FiboChip<F> {
    type Config = FiboConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
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
}

impl<F: Field> FiboInstructions<F> for FiboChip<F> {
    fn write_first_row(
        &self,
        mut layouter: impl Layouter<F>,
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
                    || Value::known(F::ONE),
                )?;

                // load f(1)
                let b_cell = region.assign_advice(
                    || "f(1) = 1",
                    self.config.b,
                    0,
                    || Value::known(F::ONE),
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

    fn write_next_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_b: AssignedCell<F, F>,
        prev_c: AssignedCell<F, F>,
        i: usize,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                // enable selector for addition
                self.config.s.enable(&mut region, i)?;

                // load prev row
                let curr_a = prev_b.copy_advice(
                    || "prev_b to curr_a",
                    &mut region,
                    self.config.a,
                    i,
                )?;

                let curr_b = prev_c.copy_advice(
                    || "prev_c to curr_b",
                    &mut region,
                    self.config.b,
                    i,
                )?;

                let curr_c = region.assign_advice(
                    || "f(2)",
                    self.config.c,
                    i,
                    || curr_a.value().copied() + curr_b.value()
                )?;

                // return
                Ok((curr_a, curr_b, curr_c))
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.i, 0)
    }
}
