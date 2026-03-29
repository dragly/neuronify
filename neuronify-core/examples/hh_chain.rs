const NUM_COMPARTMENTS: usize = 10;
const CDT: f64 = 0.01;
const STEPS: usize = 400;
const PRINT_EVERY: usize = 5;

const CAPACITANCE: f64 = 1.0;

const COUPLING_CAPACITANCE: f64 = 0.05;

const FIRE_IMPULSE_INITIAL: f64 = 800.0;
const FIRE_IMPULSE_DECAY: f64 = 5.0;

const FIRE_EVERY: usize = 0;

const RECOVERY_RATE: f64 = 10.0;

const G_NA: f64 = 120.0;
const G_K: f64 = 36.0;
const LEAK_CONDUCTANCE: f64 = 1.3;

const E_NA: f64 = 115.0;
const E_K: f64 = -12.0;
const E_M: f64 = 10.6;

#[derive(Clone)]
struct Compartment {
    voltage: f64,
    m: f64,
    h: f64,
    n: f64,
    capacitance: f64,
    injected_current: f64,
    fire_impulse: f64,
}

impl Compartment {
    fn new() -> Self {
        Self {
            voltage: 0.0,
            m: 0.05,
            h: 0.6,
            n: 0.32,
            capacitance: CAPACITANCE,
            injected_current: 0.0,
            fire_impulse: 0.0,
        }
    }
}

fn fire_compartment(comp: &mut Compartment) {
    comp.fire_impulse = FIRE_IMPULSE_INITIAL;
    comp.m = 0.05;
    comp.h = 0.6;
    comp.n = 0.32;
    comp.voltage = 0.0;
}

fn hh_step(comp: &mut Compartment) {
    let v = comp.voltage;

    let alpha_m = 0.1 * (25.0 - v) / ((2.5 - 0.1 * v).exp() - 1.0);
    let beta_m = 4.0 * (-v / 18.0).exp();
    let dm = CDT * (alpha_m * (1.0 - comp.m) - beta_m * comp.m);
    comp.m = (comp.m + dm).clamp(0.0, 1.0);

    let alpha_h = 0.07 * (-v / 20.0).exp();
    let beta_h = 1.0 / ((3.0 - 0.1 * v).exp() + 1.0);
    let dh = CDT * (alpha_h * (1.0 - comp.h) - beta_h * comp.h);
    comp.h = (comp.h + dh).clamp(0.0, 1.0);

    let alpha_n = 0.01 * (10.0 - v) / ((1.0 - 0.1 * v).exp() - 1.0);
    let beta_n = 0.125 * (-v / 80.0).exp();
    let dn = CDT * (alpha_n * (1.0 - comp.n) - beta_n * comp.n);
    comp.n = (comp.n + dn).clamp(0.0, 1.0);

    let m3 = comp.m * comp.m * comp.m;
    let n4 = comp.n * comp.n * comp.n * comp.n;
    let sodium_current = -G_NA * m3 * comp.h * (comp.voltage - E_NA);
    let potassium_current = -G_K * n4 * (comp.voltage - E_K);
    let leak_current = -LEAK_CONDUCTANCE * (comp.voltage - E_M);

    let current = sodium_current + potassium_current + leak_current + comp.injected_current;
    let delta_voltage = current / comp.capacitance;
    comp.voltage += delta_voltage * CDT;

    if comp.fire_impulse > 1.0 {
        comp.voltage += comp.fire_impulse;
        comp.fire_impulse *= (-FIRE_IMPULSE_DECAY * CDT).exp();
    }

    comp.voltage = comp.voltage.clamp(-50.0, 200.0);
    comp.injected_current -= 1.0 * comp.injected_current * CDT;

    if comp.voltage < 0.0 {
        comp.h += (0.6 - comp.h) * RECOVERY_RATE * CDT;
        comp.n += (0.32 - comp.n) * RECOVERY_RATE * CDT;
        comp.m += (0.05 - comp.m) * RECOVERY_RATE * CDT;
    }
}

fn coupling_step(compartments: &mut [Compartment]) {
    let old: Vec<Compartment> = compartments.to_vec();
    for i in 0..compartments.len() {
        if i > 0 {
            let voltage_diff = old[i - 1].voltage - old[i].voltage;
            let delta = voltage_diff / COUPLING_CAPACITANCE * CDT;
            compartments[i].voltage += delta;
            compartments[i - 1].voltage -= delta;
        }
    }
}

fn print_state(step: usize, compartments: &[Compartment]) {
    print!("step {:>4} | ", step);
    for comp in compartments {
        let v = comp.voltage;
        let ch = if v > 80.0 {
            '#'
        } else if v > 50.0 {
            '*'
        } else if v > 20.0 {
            '+'
        } else if v > 5.0 {
            '.'
        } else {
            ' '
        };
        print!("{:>7.1} {} ", v, ch);
    }
    println!();
}

fn main() {
    println!("HH Chain Simulation — {} compartments", NUM_COMPARTMENTS);
    println!(
        "Parameters: C={}, coupling_C={}, fire_impulse={}, decay={}",
        CAPACITANCE, COUPLING_CAPACITANCE, FIRE_IMPULSE_INITIAL, FIRE_IMPULSE_DECAY
    );
    println!(
        "HH: g_na={}, g_k={}, leak={}, E_na={}, E_k={}, E_m={}",
        G_NA, G_K, LEAK_CONDUCTANCE, E_NA, E_K, E_M
    );
    println!();

    print!("{:>14}", "");
    for i in 0..NUM_COMPARTMENTS {
        print!("   C{:<6}", i);
    }
    println!();
    println!("{}", "-".repeat(14 + NUM_COMPARTMENTS * 10));

    let mut compartments: Vec<Compartment> =
        (0..NUM_COMPARTMENTS).map(|_| Compartment::new()).collect();

    fire_compartment(&mut compartments[0]);

    for step in 0..STEPS {
        if FIRE_EVERY != 0 && step > 0 && step % FIRE_EVERY == 0 {
            fire_compartment(&mut compartments[0]);
            println!(
                "--- RE-FIRE at step {} (t = {:.2} ms) ---",
                step,
                step as f64 * CDT
            );
        }

        for comp in compartments.iter_mut() {
            hh_step(comp);
        }

        coupling_step(&mut compartments);

        if step % PRINT_EVERY == 0 {
            print_state(step, &compartments);
        }
    }

    println!();
    println!("Peak detection (re-running):");
    let mut compartments: Vec<Compartment> =
        (0..NUM_COMPARTMENTS).map(|_| Compartment::new()).collect();
    compartments[0].fire_impulse = FIRE_IMPULSE_INITIAL;
    let mut peaks = [(0.0_f64, 0_usize); NUM_COMPARTMENTS];

    for step in 0..STEPS {
        for comp in compartments.iter_mut() {
            hh_step(comp);
        }
        coupling_step(&mut compartments);

        for (i, comp) in compartments.iter().enumerate() {
            if comp.voltage > peaks[i].0 {
                peaks[i] = (comp.voltage, step);
            }
        }
    }

    for (i, (peak_v, peak_step)) in peaks.iter().enumerate() {
        println!(
            "  C{}: peak = {:.1} mV at step {} (t = {:.2} ms)",
            i,
            peak_v,
            peak_step,
            *peak_step as f64 * CDT
        );
    }
}
