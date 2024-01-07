use std::marker::PhantomData;
use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, Cell, Value, Chip, SimpleFloorPlanner},
    plonk::{Column, Advice, Fixed, Instance, Assigned, Error, ConstraintSystem, Circuit},
    poly::Rotation,
};

/** ***** ***** ***** ***** ***** ***** ***** ***** ***** *****
 * Config
 ***** ***** ***** ***** ***** ***** ***** ***** ***** ***** */
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Clone)]
struct TutorialConfig {
    l: Column<Advice>,
    r: Column<Advice>,
    o: Column<Advice>,

    sl: Column<Fixed>,
    sr: Column<Fixed>,
    so: Column<Fixed>,
    sm: Column<Fixed>,
    sc: Column<Fixed>,
    pi: Column<Instance>,
}

/** ***** ***** ***** ***** ***** ***** ***** ***** ***** *****
 * Instruction
 ***** ***** ***** ***** ***** ***** ***** ***** ***** ***** */
trait TutorialComposer<F: Field> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;


    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error>;
}

/** ***** ***** ***** ***** ***** ***** ***** ***** ***** *****
 * Chip
 ***** ***** ***** ***** ***** ***** ***** ***** ***** ***** */
struct TutorialChip<F: Field> {
    config: TutorialConfig,
    _maker: PhantomData<F>,
}

impl<F: Field> TutorialChip<F> {
    fn new(config: TutorialConfig) -> Self {
        Self {
            config,
            _maker: PhantomData,
        }
    }
}

impl<F: Field> Chip<F> for TutorialChip<F> {
    type Config = TutorialConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: Field> TutorialComposer<F> for TutorialChip<F> {
    fn raw_multiply<FM>(
            &self,
            layouter: &mut impl Layouter<F>,
            mut f: FM,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)> {
        layouter.assign_region(
            || "multiply",
            |mut region| {
                let mut values = None;
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|v| v.0)
                    },
                )?;

                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|v| v.1),
                )?;

                let out = region.assign_advice(
                    || "rhs",
                    self.config.o,
                    0,
                    || values.unwrap().map(|v| v.2),
                )?;

                region.assign_fixed(|| "sm", self.config.sm, 0, || Value::known(F::ONE))?;
                region.assign_fixed(|| "so", self.config.so, 0, || Value::known(F::ONE))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn raw_add<FM>(
            &self,
            layouter: &mut impl Layouter<F>,
            mut f: FM,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)> {
        layouter.assign_region(
            || "add",
            |mut region| {
                let mut values = None;
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|v| v.0)
                    },
                )?;

                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|v| v.1),
                )?;

                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|v| v.2),
                )?;

                region.assign_fixed(|| "sl", self.config.sl, 0, || Value::known(F::ONE))?;
                region.assign_fixed(|| "sr", self.config.sr, 0, || Value::known(F::ONE))?;
                region.assign_fixed(|| "so", self.config.so, 0, || Value::known(F::ONE))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            }
        )
    }

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error> {
        layouter.assign_region(
            || "copy",
            |mut region| {
                region.constrain_equal(a, b)
            },
        )
    }

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell, self.config.pi, row)
    }
}

/** ***** ***** ***** ***** ***** ***** ***** ***** ***** *****
 * Circuit
 ***** ***** ***** ***** ***** ***** ***** ***** ***** ***** */
#[derive(Default)]
struct TutorialCircuit<F: Field> {
    x: Value<F>,
    y: Value<F>,
    constant: F,
}

impl<F: Field> Circuit<F> for TutorialCircuit<F> {
    type Config = TutorialConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let l = meta.advice_column();
        let r = meta.advice_column();
        let o = meta.advice_column();

        meta.enable_equality(l);
        meta.enable_equality(r);
        meta.enable_equality(o);

        let sm = meta.fixed_column();
        let sl = meta.fixed_column();
        let sr = meta.fixed_column();
        let so = meta.fixed_column();
        let sc = meta.fixed_column();

        let pi = meta.instance_column();
        meta.enable_equality(pi);

        meta.create_gate("mini plonk", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let sl = meta.query_fixed(sl, Rotation::cur());
            let sr = meta.query_fixed(sr, Rotation::cur());
            let so = meta.query_fixed(so, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());

            vec![l.clone() * sl + r.clone() * sr + l * r * sm + (o * so * (-F::ONE)) + sc]
        });

        TutorialConfig {
            l,
            r,
            o,
            sl,
            sr,
            so,
            sm,
            sc,
            pi,
        }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = TutorialChip::new(config);

        let x: Value<Assigned<_>> = self.x.into();
        let y: Value<Assigned<_>> = self.y.into();
        let consty = Assigned::from(self.constant);

        // x^2
        let (a0, b0, c0) = chip.raw_multiply(&mut layouter, || x.map(|x| (x, x, x * x)))?;
        chip.copy(&mut layouter, a0, b0)?;

        // y^2
        let (a1, b1, c1) = chip.raw_multiply(&mut layouter, || y.map(|y| (y, y, y * y)))?;
        chip.copy(&mut layouter, a1, b1)?;

        // x^2 * y^2
        let (a2, b2, c2) = chip.raw_multiply(&mut layouter, || {
            x.zip(y).map(|(x, y)| (x * x, y * y, x * x * y * y))
        })?;
        chip.copy(&mut layouter, c0, a2)?;
        chip.copy(&mut layouter, c1, b2)?;

        let (a3, b3, c3) = chip.raw_add(&mut layouter, || {
            x.zip(y)
                .map(|(x, y)| (x * x * y * y, consty, x * x * y * y + consty))
        })?;
        chip.copy(&mut layouter, c2, a3)?;

        chip.expose_public(&mut layouter, b3, 0)?;
        layouter.constrain_instance(c3, chip.config.pi, 1)?;

        Ok(())
    }
}

fn main() {
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr as Fp;

    let k = 4;
    let constant = Fp::from(7);
    let x = Fp::from(5);
    let y = Fp::from(9);
    let z = Fp::from(25 * 81 + 7);

    let circuit: TutorialCircuit<Fp> = TutorialCircuit {
        x: Value::known(x),
        y: Value::known(y),
        constant,
    };

    let public_inputs = vec![constant, z];

    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
