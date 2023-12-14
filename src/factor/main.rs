use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Value, SimpleFloorPlanner, Layouter},
    plonk::{Advice, Column, Circuit, ConstraintSystem, Instance, Selector, Error},
    poly::Rotation,
};

// A variable representing a number.
#[derive(Clone)]
struct Number<F: Field>(AssignedCell<F, F>);

/** ***** ***** ***** ***** *****
 * Config
 ***** ***** ***** ***** ***** */
#[derive(Clone)]
struct FactorConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    instance: Column<Instance>,
    s: Selector,
}

/** ***** ***** ***** ***** *****
 * Circuit
 ***** ***** ***** ***** ***** */
#[derive(Default)]
struct FactorCircuit<F> {
    a: F,
    b: F,
}

impl<F: Field> Circuit<F> for FactorCircuit<F> {
    type Config = FactorConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // create columns
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let instance = meta.instance_column();
        let s = meta.selector();

        // enable permutation checks for the following columns
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(instance);

        // define custom gate
        meta.create_gate("mul", |meta| {
            let aa = meta.query_advice(a, Rotation::cur());
            let bb = meta.query_advice(b, Rotation::cur());
            let cc = meta.query_advice(c, Rotation::cur());
            let ss = meta.query_selector(s);
            vec![ss * (aa * bb - cc)]
        });

        // return
        FactorConfig {
            a, b, c, instance, s
        }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let c = layouter.assign_region(
            || "assign the row",
            |mut region| {
                // selector enable
                config.s.enable(&mut region, 0)?;

                // load a
                region.assign_advice(|| "load a", config.a, 0, || Value::known(self.a))?;

                // load b
                region.assign_advice(|| "load b", config.b, 0, || Value::known(self.b))?;

                // load c
                let c_cell = region
                    .assign_advice_from_instance(|| "load c", config.instance, 0, config.c, 0)
                    .map(Number)?;
                Ok(c_cell)
            },
        )?;
        layouter.constrain_instance(c.0.cell(), config.instance, 0)?;
        Ok(())
    }
}

fn main() {
    use halo2_proofs::dev::MockProver;
    use halo2curves::pasta::Fp;

    let a_input = 11;
    let b_input = 13;
    let c_input = 11 * 13;

    let circuit = FactorCircuit {
        a: Fp::from(a_input),
        b: Fp::from(b_input),
    };

    let public_inputs = vec![Fp::from(c_input)];

    let k = 4;

    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
    println!("Success");
}
