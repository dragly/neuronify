use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

pub trait Quantity: Copy + PartialOrd {
    fn value(self) -> f64;
    fn value_mut(&mut self) -> &mut f64;
}

impl Quantity for f64 {
    fn value(self) -> f64 {
        self
    }

    fn value_mut(&mut self) -> &mut f64 {
        self
    }
}

macro_rules! units {
    ($($name:ident: $symbol:literal),* $(,)?) => {$(
        #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Deserialize, Serialize)]
        #[serde(transparent)]
        #[repr(transparent)]
        pub struct $name(pub f64);

        impl $name {
            pub const ZERO: $name = $name(0.0);
            pub const INFINITY: $name = $name(f64::INFINITY);
            pub const NEG_INFINITY: $name = $name(f64::NEG_INFINITY);

            pub fn abs(self) -> $name {
                $name(self.0.abs())
            }

            pub fn min(self, other: $name) -> $name {
                $name(self.0.min(other.0))
            }

            pub fn max(self, other: $name) -> $name {
                $name(self.0.max(other.0))
            }

            pub fn clamp(self, min: $name, max: $name) -> $name {
                $name(self.0.clamp(min.0, max.0))
            }

            pub fn is_finite(self) -> bool {
                self.0.is_finite()
            }

            pub fn total_cmp(&self, other: &$name) -> Ordering {
                self.0.total_cmp(&other.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self.0, formatter)?;
                formatter.write_str(concat!(" ", $symbol))
            }
        }

        impl Quantity for $name {
            fn value(self) -> f64 {
                self.0
            }

            fn value_mut(&mut self) -> &mut f64 {
                &mut self.0
            }
        }

        impl Add for $name {
            type Output = $name;
            fn add(self, other: $name) -> $name {
                $name(self.0 + other.0)
            }
        }

        impl Sub for $name {
            type Output = $name;
            fn sub(self, other: $name) -> $name {
                $name(self.0 - other.0)
            }
        }

        impl Neg for $name {
            type Output = $name;
            fn neg(self) -> $name {
                $name(-self.0)
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, other: $name) {
                self.0 += other.0;
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, other: $name) {
                self.0 -= other.0;
            }
        }

        impl Mul<f64> for $name {
            type Output = $name;
            fn mul(self, factor: f64) -> $name {
                $name(self.0 * factor)
            }
        }

        impl Mul<$name> for f64 {
            type Output = $name;
            fn mul(self, quantity: $name) -> $name {
                $name(self * quantity.0)
            }
        }

        impl MulAssign<f64> for $name {
            fn mul_assign(&mut self, factor: f64) {
                self.0 *= factor;
            }
        }

        impl Div<f64> for $name {
            type Output = $name;
            fn div(self, divisor: f64) -> $name {
                $name(self.0 / divisor)
            }
        }

        impl Div for $name {
            type Output = f64;
            fn div(self, other: $name) -> f64 {
                self.0 / other.0
            }
        }

        impl Sum for $name {
            fn sum<I: Iterator<Item = $name>>(iter: I) -> $name {
                $name(iter.map(|quantity| quantity.0).sum())
            }
        }

        impl<'a> Sum<&'a $name> for $name {
            fn sum<I: Iterator<Item = &'a $name>>(iter: I) -> $name {
                $name(iter.map(|quantity| quantity.0).sum())
            }
        }
    )*};
}
macro_rules! product {
    ($first:ident * $second:ident = $product:ident) => {
        impl Mul<$second> for $first {
            type Output = $product;
            fn mul(self, other: $second) -> $product {
                $product(self.0 * other.0)
            }
        }

        impl Mul<$first> for $second {
            type Output = $product;
            fn mul(self, other: $first) -> $product {
                $product(self.0 * other.0)
            }
        }

        impl Div<$first> for $product {
            type Output = $second;
            fn div(self, other: $first) -> $second {
                $second(self.0 / other.0)
            }
        }

        impl Div<$second> for $product {
            type Output = $first;
            fn div(self, other: $second) -> $first {
                $first(self.0 / other.0)
            }
        }
    };
}
macro_rules! reciprocal {
    ($first:ident, $second:ident) => {
        impl Mul<$second> for $first {
            type Output = f64;
            fn mul(self, other: $second) -> f64 {
                self.0 * other.0
            }
        }

        impl Mul<$first> for $second {
            type Output = f64;
            fn mul(self, other: $first) -> f64 {
                self.0 * other.0
            }
        }

        impl Div<$first> for f64 {
            type Output = $second;
            fn div(self, other: $first) -> $second {
                $second(self / other.0)
            }
        }

        impl Div<$second> for f64 {
            type Output = $first;
            fn div(self, other: $second) -> $first {
                $first(self / other.0)
            }
        }
    };
}
macro_rules! over_area {
    ($density:ident * SquareMicrometres = $total:ident, $factor:expr) => {
        impl Mul<SquareMicrometres> for $density {
            type Output = $total;
            fn mul(self, area: SquareMicrometres) -> $total {
                $total(self.0 * area.0 * $factor)
            }
        }
    };
}

units!(
    Millivolts: "mV",
    Milliseconds: "ms",
    Seconds: "s",
    Nanoamperes: "nA",
    Microsiemens: "µS",
    Nanofarads: "nF",
    Megaohms: "MΩ",
    Kilohertz: "kHz",
    Hertz: "Hz",
    Micrometres: "µm",
    SquareMicrometres: "µm²",
    Celsius: "°C",
    PerMillivolt: "/mV",
    NanoamperesPerMillisecond: "nA/ms",
    Picocoulombs: "pC",
    SiemensPerSquareCentimetre: "S/cm²",
    MilliamperesPerSquareCentimetre: "mA/cm²",
    MicrofaradsPerSquareCentimetre: "µF/cm²",
    OhmSquareCentimetres: "Ω·cm²",
    OhmCentimetres: "Ω·cm",
);

product!(Microsiemens * Millivolts = Nanoamperes);
product!(Megaohms * Nanoamperes = Millivolts);
product!(Microsiemens * Milliseconds = Nanofarads);
product!(Megaohms * Nanofarads = Milliseconds);
product!(Nanoamperes * Milliseconds = Picocoulombs);
product!(NanoamperesPerMillisecond * Milliseconds = Nanoamperes);
product!(SiemensPerSquareCentimetre * Millivolts = MilliamperesPerSquareCentimetre);

reciprocal!(Microsiemens, Megaohms);
reciprocal!(Kilohertz, Milliseconds);
reciprocal!(Hertz, Seconds);
reciprocal!(Millivolts, PerMillivolt);
reciprocal!(SiemensPerSquareCentimetre, OhmSquareCentimetres);

over_area!(
    SiemensPerSquareCentimetre * SquareMicrometres = Microsiemens,
    1e-2
);
over_area!(
    MicrofaradsPerSquareCentimetre * SquareMicrometres = Nanofarads,
    1e-5
);

impl Milliseconds {
    pub fn to_seconds(self) -> Seconds {
        Seconds(self.0 * 1e-3)
    }
}

impl Seconds {
    pub fn to_milliseconds(self) -> Milliseconds {
        Milliseconds(self.0 * 1e3)
    }
}

impl Kilohertz {
    pub fn to_hertz(self) -> Hertz {
        Hertz(self.0 * 1e3)
    }
}

impl Hertz {
    pub fn to_kilohertz(self) -> Kilohertz {
        Kilohertz(self.0 * 1e-3)
    }

    pub fn period(self) -> Milliseconds {
        Milliseconds(1e3 / self.0)
    }

    pub fn cycles_in(self, duration: Milliseconds) -> f64 {
        self * duration.to_seconds()
    }
}

impl Micrometres {
    pub fn squared(self) -> SquareMicrometres {
        SquareMicrometres(self.0.powi(2))
    }
}

impl Mul for Micrometres {
    type Output = SquareMicrometres;
    fn mul(self, other: Micrometres) -> SquareMicrometres {
        SquareMicrometres(self.0 * other.0)
    }
}

impl SquareMicrometres {
    pub fn sqrt(self) -> Micrometres {
        Micrometres(self.0.sqrt())
    }
}

impl Div<MicrofaradsPerSquareCentimetre> for Nanofarads {
    type Output = SquareMicrometres;
    fn div(self, capacitance: MicrofaradsPerSquareCentimetre) -> SquareMicrometres {
        SquareMicrometres(self.0 * 1e5 / capacitance.0)
    }
}

impl OhmCentimetres {
    pub fn along(self, length: Micrometres, cross_section: SquareMicrometres) -> Megaohms {
        Megaohms(self.0 * length.0 / cross_section.0 * 1e-2)
    }
}

impl Nanofarads {
    pub fn in_picofarads(self) -> f64 {
        self.0 * 1e3
    }
}

impl Nanoamperes {
    pub fn in_picoamperes(self) -> f64 {
        self.0 * 1e3
    }
}

impl Microsiemens {
    pub fn in_nanosiemens(self) -> f64 {
        self.0 * 1e3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_units_multiply_without_conversion_factors() {
        assert_eq!(Microsiemens(0.5) * Millivolts(-70.0), Nanoamperes(-35.0));
        assert_eq!(Nanoamperes(2.0) * Megaohms(100.0), Millivolts(200.0));
        assert_eq!(Megaohms(100.0) * Nanofarads(0.2), Milliseconds(20.0));
        assert_eq!(Nanofarads(0.2) / Milliseconds(0.025), Microsiemens(8.0));
        assert_eq!(1.0 / Megaohms(100.0), Microsiemens(0.01));
        assert_eq!(Kilohertz(0.02) * Milliseconds(50.0), 1.0);
    }

    fn close(first: f64, second: f64) -> bool {
        (first - second).abs() <= 1e-12 * second.abs()
    }

    #[test]
    fn densities_over_an_area_give_engine_units() {
        let area = SquareMicrometres(1e8);
        assert!(close((SiemensPerSquareCentimetre(0.12) * area).0, 1.2e5));
        assert!(close((MicrofaradsPerSquareCentimetre(1.0) * area).0, 1e3));
        assert!(close(
            (Nanofarads(1e3) / MicrofaradsPerSquareCentimetre(1.0)).0,
            1e8
        ));
        let resistance = OhmCentimetres(100.0).along(Micrometres(1e4), SquareMicrometres(1e8));
        assert!(close(resistance.0, 1e-4));
    }

    #[test]
    fn quantities_print_with_their_unit() {
        assert_eq!(format!("{:.1}", Millivolts(-65.04)), "-65.0 mV");
        assert_eq!(Milliseconds(0.025).to_string(), "0.025 ms");
    }

    #[test]
    fn rates_and_clocks_convert_between_seconds_and_milliseconds() {
        assert_eq!(Hertz(20.0).period(), Milliseconds(50.0));
        assert_eq!(Hertz(20.0).to_kilohertz(), Kilohertz(0.02));
        assert_eq!(Hertz(10.0).cycles_in(Milliseconds(500.0)), 5.0);
        assert_eq!(Seconds(0.25).to_milliseconds(), Milliseconds(250.0));
        assert_eq!(Milliseconds(250.0).to_seconds(), Seconds(0.25));
    }
}
