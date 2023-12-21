use halo2_proofs::{
    arithmetic::Field,
    circuit::{SimpleFloorPlanner, Layouter},
    plonk::{Circuit, ConstraintSystem, Error},
};
use crate::chip::FiboChip;
use crate::config::FiboConfig;

#[derive(Default)]
pub struct FiboCircuit {
    pub n: usize,
}

impl<F: Field> Circuit<F> for FiboCircuit {
    type Config = FiboConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let i = meta.instance_column();
        let s = meta.selector();

        FiboChip::configure(meta, [a, b, c], i, s)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FiboChip::construct(config);

        // first row
        let (_, mut b, mut c) = chip.load_first_row(layouter.namespace(|| "first"))?;

        // next row
        for i in 1..=self.n - 2 {
            (_, b, c) = chip.load_row(layouter.namespace(|| "next"), b, c, i)?;
        }

        // expose
        chip.expose_public(layouter.namespace(|| "expose"), &c)?;

        Ok(())
    }
}
