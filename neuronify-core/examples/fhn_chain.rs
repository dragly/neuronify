/// FitzHugh-Nagumo compartment chain simulation.
/// Run with: cargo run -p neuronify-core --example fhn_chain
///
/// FHN is a 2-variable simplification of HH that produces sharp action
/// potentials with a natural refractory period — no if-tests needed.

// ─── Tunable parameters ───────────────────────────────────────────

const NUM_COMPARTMENTS: usize = 10;
const DT: f64 = 0.01;
const STEPS: usize = 1500;
const PRINT_EVERY: usize = 5;

// FHN dynamics
const TAU: f64 = 60.0;  // speed of voltage dynamics (higher = sharper AP, max ~65 for stability)
const A: f64 = 0.7;
const B: f64 = 0.8;
const EPSILON: f64 = 0.08; // recovery speed: smaller = longer refractory

// Inter-compartment coupling
const COUPLING: f64 = 24.0;

// Fire: direct voltage kick above threshold
const FIRE_V: f64 = 1.0; // FHN threshold is around v ≈ -0.4, so v=1.0 is well above

// Repeated firing interval (0 = fire only once)
const FIRE_EVERY: usize = 200;

// Voltage scaling for display (FHN v is roughly in [-2, 2])
const V_SCALE: f64 = 50.0;
const V_OFFSET: f64 = 50.0;

// ─── Compartment state ────────────────────────────────────────────

#[derive(Clone)]
struct Compartment {
    v: f64,    // fast voltage variable
    w: f64,    // slow recovery variable
}

impl Compartment {
    fn new() -> Self {
        // Resting state of FHN (approximate fixed point for a=0.7, b=0.8)
        Self {
            v: -1.2,
            w: -0.625,
        }
    }

    fn fire(&mut self) {
        self.v = FIRE_V;
    }

    fn display_voltage(&self) -> f64 {
        self.v * V_SCALE + V_OFFSET
    }
}

// ─── Simulation ───────────────────────────────────────────────────

fn fhn_step(comp: &mut Compartment) {
    let v = comp.v;
    let w = comp.w;

    // FitzHugh-Nagumo equations (TAU scales the fast variable speed)
    let dv = TAU * (v - v * v * v / 3.0 - w);
    let dw = TAU * EPSILON * (v + A - B * w);

    comp.v += dv * DT;
    comp.w += dw * DT;
}

fn coupling_step(compartments: &mut [Compartment]) {
    let old: Vec<f64> = compartments.iter().map(|c| c.v).collect();
    for i in 0..compartments.len() {
        if i > 0 {
            let diff = old[i - 1] - old[i];
            compartments[i].v += COUPLING * diff * DT;
            compartments[i - 1].v -= COUPLING * diff * DT;
        }
    }
}

fn print_state(step: usize, compartments: &[Compartment]) {
    print!("step {:>4} | ", step);
    for comp in compartments {
        let dv = comp.display_voltage();
        let ch = if dv > 80.0 {
            '#'
        } else if dv > 50.0 {
            '*'
        } else if dv > 20.0 {
            '+'
        } else if dv > 5.0 {
            '.'
        } else {
            ' '
        };
        print!("{:>7.1} {} ", dv, ch);
    }
    println!();
}

fn main() {
    println!("FHN Chain Simulation — {} compartments", NUM_COMPARTMENTS);
    println!(
        "Parameters: a={}, b={}, eps={}, coupling={}, fire_v={}",
        A, B, EPSILON, COUPLING, FIRE_V
    );
    println!();

    // Header
    print!("{:>14}", "");
    for i in 0..NUM_COMPARTMENTS {
        print!("   C{:<6}", i);
    }
    println!();
    println!("{}", "-".repeat(14 + NUM_COMPARTMENTS * 10));

    let mut compartments: Vec<Compartment> =
        (0..NUM_COMPARTMENTS).map(|_| Compartment::new()).collect();

    // Fire the first compartment
    compartments[0].fire();

    for step in 0..STEPS {
        if FIRE_EVERY > 0 && step > 0 && step % FIRE_EVERY == 0 {
            compartments[0].fire();
            println!(
                "--- RE-FIRE at step {} (t = {:.2}) ---",
                step,
                step as f64 * DT
            );
        }

        for comp in compartments.iter_mut() {
            fhn_step(comp);
        }
        coupling_step(&mut compartments);

        if step % PRINT_EVERY == 0 {
            print_state(step, &compartments);
        }
    }

    // Peak detection
    println!();
    println!("Peak detection (re-running):");
    let mut compartments: Vec<Compartment> =
        (0..NUM_COMPARTMENTS).map(|_| Compartment::new()).collect();
    compartments[0].fire();
    let mut peaks = vec![(f64::MIN, 0_usize); NUM_COMPARTMENTS];

    for step in 0..STEPS {
        for comp in compartments.iter_mut() {
            fhn_step(comp);
        }
        coupling_step(&mut compartments);

        for (i, comp) in compartments.iter().enumerate() {
            if comp.display_voltage() > peaks[i].0 {
                peaks[i] = (comp.display_voltage(), step);
            }
        }
    }

    for (i, (peak_v, peak_step)) in peaks.iter().enumerate() {
        println!(
            "  C{}: peak = {:.1} at step {} (t = {:.2})",
            i,
            peak_v,
            peak_step,
            *peak_step as f64 * DT
        );
    }
}
