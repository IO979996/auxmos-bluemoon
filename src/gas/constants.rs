use bitflags::bitflags;

/// kPa*L/(K*mol)
pub const R_IDEAL_GAS_EQUATION: f32 = 8.31;
/// kPa
pub const ONE_ATMOSPHERE: f32 = 101.325;
///  -270.3degC
pub const TCMB: f32 = 2.7;
///  -48.15degC
pub const TCRYO: f32 = 225.0;
///  0degC
pub const T0C: f32 = 273.15;
///  20degC
pub const T20C: f32 = 293.15;
/// Amount of gas below which any amounts will be truncated to 0.
pub const GAS_MIN_MOLES: f32 = 0.0001;
/// Heat capacities below which heat will be considered 0.
pub const MINIMUM_HEAT_CAPACITY: f32 = 0.0003;

/// liters in a cell
pub const CELL_VOLUME: f32 = 2500.0;
/// moles in a 2.5 m^3 cell at 101.325 Pa and 20 degC
pub const MOLES_CELLSTANDARD: f32 = ONE_ATMOSPHERE * CELL_VOLUME / (T20C * R_IDEAL_GAS_EQUATION);
/// compared against for superconductivity
pub const M_CELL_WITH_RATIO: f32 = MOLES_CELLSTANDARD * 0.005;
/// percentage of oxygen in a normal mixture of air
pub const O2STANDARD: f32 = 0.21;
/// same but for nitrogen
pub const N2STANDARD: f32 = 0.79;
///  O2 standard value (21%)
pub const MOLES_O2STANDARD: f32 = MOLES_CELLSTANDARD * O2STANDARD;
///  N2 standard value (79%)
pub const MOLES_N2STANDARD: f32 = MOLES_CELLSTANDARD * N2STANDARD;
/// liters in a normal breath
pub const BREATH_VOLUME: f32 = 0.5;
/// Amount of air to take a from a tile
pub const BREATH_PERCENTAGE: f32 = BREATH_VOLUME / CELL_VOLUME;

/// EXCITED GROUPS

/// number of FULL air controller ticks before an excited group breaks down (averages gas contents across turfs)
pub const EXCITED_GROUP_BREAKDOWN_CYCLES: i32 = 4;
/// number of FULL air controller ticks before an excited group dismantles and removes its turfs from active
pub const EXCITED_GROUP_DISMANTLE_CYCLES: i32 = 16;

/// Ratio of air that must move to/from a tile to reset group processing
pub const MINIMUM_AIR_RATIO_TO_SUSPEND: f32 = 0.1;
/// Minimum ratio of air that must move to/from a tile
pub const MINIMUM_AIR_RATIO_TO_MOVE: f32 = 0.001;
/// Minimum amount of air that has to move before a group processing can be suspended
pub const MINIMUM_AIR_TO_SUSPEND: f32 = MOLES_CELLSTANDARD * MINIMUM_AIR_RATIO_TO_SUSPEND;
/// Either this must be active
pub const MINIMUM_MOLES_DELTA_TO_MOVE: f32 = MOLES_CELLSTANDARD * MINIMUM_AIR_RATIO_TO_MOVE;
/// or this (or both, obviously)
pub const MINIMUM_TEMPERATURE_TO_MOVE: f32 = T20C + 100.0;
/// Minimum temperature difference before group processing is suspended
pub const MINIMUM_TEMPERATURE_DELTA_TO_SUSPEND: f32 = 4.0;
/// Minimum temperature difference before the gas temperatures are just set to be equal
pub const MINIMUM_TEMPERATURE_DELTA_TO_CONSIDER: f32 = 0.5;
pub const MINIMUM_TEMPERATURE_FOR_SUPERCONDUCTION: f32 = T20C + 10.0;
pub const MINIMUM_TEMPERATURE_START_SUPERCONDUCTION: f32 = T20C + 200.0;

/// The amount of gas that is diffused between tiles every tick. Must be less than 1/6.
pub const GAS_DIFFUSION_CONSTANT: f32 = 0.125;

/// This number minus the number of adjacent turfs is how much the original gas needs to be multiplied by to represent loss by diffusion
pub const GAS_LOSS_CONSTANT: f32 = 1.0 / GAS_DIFFUSION_CONSTANT;

/// HEAT TRANSFER COEFFICIENTS

/// Must be between 0 and 1. Values closer to 1 equalize temperature faster

/// Should not exceed 0.4 else the algorithm will diverge

pub const WALL_HEAT_TRANSFER_COEFFICIENT: f32 = 0.0;
pub const OPEN_HEAT_TRANSFER_COEFFICIENT: f32 = 0.4;
/// a hack for now
pub const WINDOW_HEAT_TRANSFER_COEFFICIENT: f32 = 0.1;
/// a hack to help make vacuums "cold", sacrificing realism for gameplay
pub const HEAT_CAPACITY_VACUUM: f32 = 7000.0;

/// The Stefan-Boltzmann constant. M T^-3 Θ^-4
pub const STEFAN_BOLTZMANN_CONSTANT: f64 = 5.670_373e-08; // watts/(meter^2*kelvin^4)

const SPACE_TEMP: f64 = TCMB as f64;

/// How much power is coming in from space per square meter. M T^-3
pub const RADIATION_FROM_SPACE: f64 =
	STEFAN_BOLTZMANN_CONSTANT * SPACE_TEMP * SPACE_TEMP * SPACE_TEMP * SPACE_TEMP; // watts/meter^2

/// FIRE

pub const FIRE_MINIMUM_TEMPERATURE_TO_SPREAD: f32 = 150.0 + T0C;
pub const FIRE_MINIMUM_TEMPERATURE_TO_EXIST: f32 = 100.0 + T0C;
pub const FIRE_SPREAD_RADIOSITY_SCALE: f32 = 0.85;
/// For small fires
pub const FIRE_GROWTH_RATE: f32 = 40000.0;
pub const PLASMA_MINIMUM_BURN_TEMPERATURE: f32 = 100.0 + T0C;
pub const PLASMA_UPPER_TEMPERATURE: f32 = 1370.0 + T0C;
pub const PLASMA_OXYGEN_FULLBURN: f32 = 10.0;
pub const FIRE_MAXIMUM_BURN_RATE: f32 = 0.2;

/// GASES

pub const MIN_TOXIC_GAS_DAMAGE: i32 = 1;
pub const MAX_TOXIC_GAS_DAMAGE: i32 = 10;
/// Moles in a standard cell after which gases are visible
pub const MOLES_GAS_VISIBLE: f32 = 0.25;

/// `moles_visible` * `FACTOR_GAS_VISIBLE_MAX` = Moles after which gas is at maximum visibility
pub const FACTOR_GAS_VISIBLE_MAX: f32 = 20.0;
/// Mole step for alpha updates. This means alpha can update at 0.25, 0.5, 0.75 and so on
pub const MOLES_GAS_VISIBLE_STEP: f32 = 0.25;

/// REACTIONS

// Maximum amount of ReactionIdentifiers in the TinyVec that all_reactions returns.
// We can't guarantee the max number of reactions that will ever be registered,
// so this is here to prevent that from getting out of control.
// TinyVec is used mostly to prevent too much heap stuff from going on, since there can be a LOT of reactions going.
// ReactionIdentifier is 12 bytes, so this can be pretty generous.
pub const MAX_REACTION_TINYVEC_SIZE: usize = 32;

bitflags! {
	/// return values for reactions (bitflags)
	pub struct ReactionReturn: u32 {
		const NO_REACTION = 0b0;
		const REACTING = 0b1;
		const STOP_REACTIONS = 0b10;
	}
}

pub const GAS_O2: &str = "o2";
pub const GAS_N2: &str = "n2";
pub const GAS_CO2: &str = "co2";
pub const GAS_PLASMA: &str = "plasma";
pub const GAS_H2O: &str = "water_vapor";
pub const GAS_HYPERNOB: &str = "nob";
pub const GAS_NITROUS: &str = "n2o";
pub const GAS_NITRYL: &str = "no2";
pub const GAS_TRITIUM: &str = "tritium";
pub const GAS_BZ: &str = "bz";
pub const GAS_STIMULUM: &str = "stim";
pub const GAS_PLUOXIUM: &str = "pluox";
pub const GAS_MIASMA: &str = "miasma";
pub const GAS_METHANE: &str = "methane";
pub const GAS_METHYL_BROMIDE: &str = "methyl_bromide";
// BlueMoon-Station: дополнительные газы (ID совпадают с code/__DEFINES/atmospherics.dm)
pub const GAS_NITRIC: &str = "no";
pub const GAS_BROMINE: &str = "bromine";
pub const GAS_AMMONIA: &str = "ammonia";
pub const GAS_FLUORINE: &str = "fluorine";
pub const GAS_ETHANOL: &str = "ethanol";
pub const GAS_MOTOR_OIL: &str = "motor_oil";
pub const GAS_QCD: &str = "qcd";
pub const GAS_HELIUM: &str = "helium";
pub const GAS_FREON: &str = "freon";
pub const GAS_HALON: &str = "halon";
pub const GAS_ANTINOBLIUM: &str = "antinoblium";
pub const GAS_PROTO_NITRATE: &str = "proto_nitrate";
pub const GAS_ZAUKER: &str = "zauker";
pub const GAS_HEALIUM: &str = "healium";
pub const GAS_NITRIUM: &str = "nitrium";
pub const GAS_HYDROGEN: &str = "hydrogen";

// BlueMoon reaction constants (from __DEFINES/reactions.dm)
pub const WATER_VAPOR_FREEZE: f32 = 200.0;
pub const NITRYL_FORMATION_ENERGY: f32 = 100_000.0;
pub const FIRE_CARBON_ENERGY_RELEASED: f32 = 100_000.0;
pub const STIMULUM_HEAT_SCALE: f32 = 100_000.0;
pub const STIMULUM_FIRST_RISE: f32 = 0.65;
pub const STIMULUM_FIRST_DROP: f32 = 0.065;
pub const STIMULUM_SECOND_RISE: f32 = 0.0009;
pub const STIMULUM_ABSOLUTE_DROP: f32 = 0.000_003_35;
pub const REACTION_OPPRESSION_THRESHOLD: f32 = 5.0;
pub const NOBLIUM_FORMATION_ENERGY: f32 = 2e9;
pub const NOBLIUM_FORMATION_MAX_TEMP: f32 = 15.0;
pub const BZ_RESEARCH_SCALE: f32 = 4.0;
pub const BZ_RESEARCH_MAX_AMOUNT: f32 = 400.0;
pub const STIMULUM_RESEARCH_AMOUNT: f32 = 50.0;
pub const NOBLIUM_RESEARCH_AMOUNT: f32 = 25.0;
pub const MIASMA_RESEARCH_AMOUNT: f32 = 6.0;
pub const QCD_RESEARCH_AMOUNT: f32 = 0.2;
pub const FREON_MAXIMUM_BURN_TEMPERATURE: f32 = T0C;
pub const FREON_CATALYST_MAX_TEMPERATURE: f32 = 310.0;
pub const FREON_LOWER_TEMPERATURE: f32 = 60.0;
pub const FREON_TERMINAL_TEMPERATURE: f32 = 50.0;
pub const FREON_HOT_ICE_MIN_TEMP: f32 = 120.0;
pub const FREON_HOT_ICE_MAX_TEMP: f32 = 160.0;
pub const FREON_OXYGEN_FULLBURN: f32 = 10.0;
pub const FREON_BURN_RATE_DELTA: f32 = 4.0;
pub const FIRE_FREON_ENERGY_CONSUMED: f32 = 3e5;
pub const FREON_FORMATION_MIN_TEMPERATURE: f32 = FIRE_MINIMUM_TEMPERATURE_TO_EXIST + 100.0;
pub const FREON_FORMATION_ENERGY_CONSUMED: f32 = 2e5;
pub const OXYGEN_BURN_RATIO_BASE: f32 = 2.0;
pub const HALON_COMBUSTION_ENERGY: f32 = 2500.0;
pub const HALON_COMBUSTION_MIN_TEMPERATURE: f32 = T0C + 70.0;
pub const HALON_COMBUSTION_TEMPERATURE_SCALE: f32 = FIRE_MINIMUM_TEMPERATURE_TO_EXIST * 10.0;
pub const HEALIUM_FORMATION_MIN_TEMP: f32 = 25.0;
pub const HEALIUM_FORMATION_MAX_TEMP: f32 = 300.0;
pub const HEALIUM_FORMATION_ENERGY: f32 = 9000.0;
pub const ZAUKER_FORMATION_MIN_TEMPERATURE: f32 = 50_000.0;
pub const ZAUKER_FORMATION_MAX_TEMPERATURE: f32 = 75_000.0;
pub const ZAUKER_FORMATION_TEMPERATURE_SCALE: f32 = 5e-6;
pub const ZAUKER_FORMATION_ENERGY: f32 = 5000.0;
pub const ZAUKER_DECOMPOSITION_MAX_RATE: f32 = 20.0;
pub const ZAUKER_DECOMPOSITION_ENERGY: f32 = 460.0;
pub const NITRIUM_FORMATION_MIN_TEMP: f32 = 1500.0;
pub const NITRIUM_FORMATION_TEMP_DIVISOR: f32 = FIRE_MINIMUM_TEMPERATURE_TO_EXIST * 8.0;
pub const NITRIUM_FORMATION_ENERGY: f32 = 100_000.0;
pub const NITRIUM_DECOMPOSITION_MAX_TEMP: f32 = T0C + 70.0;
pub const NITRIUM_DECOMPOSITION_TEMP_DIVISOR: f32 = FIRE_MINIMUM_TEMPERATURE_TO_EXIST * 8.0;
pub const NITRIUM_DECOMPOSITION_ENERGY: f32 = 30_000.0;
pub const PLUOXIUM_FORMATION_MIN_TEMP: f32 = 50.0;
pub const PLUOXIUM_FORMATION_MAX_TEMP: f32 = T0C;
pub const PLUOXIUM_FORMATION_MAX_RATE: f32 = 5.0;
pub const PLUOXIUM_FORMATION_ENERGY: f32 = 250.0;
pub const PN_FORMATION_MIN_TEMPERATURE: f32 = 5000.0;
pub const PN_FORMATION_MAX_TEMPERATURE: f32 = 10_000.0;
pub const PN_FORMATION_ENERGY: f32 = 650.0;
pub const PN_HYDROGEN_CONVERSION_THRESHOLD: f32 = 150.0;
pub const PN_HYDROGEN_CONVERSION_MAX_RATE: f32 = 5.0;
pub const PN_HYDROGEN_CONVERSION_ENERGY: f32 = 2500.0;
pub const PN_TRITIUM_CONVERSION_MIN_TEMP: f32 = 150.0;
pub const PN_TRITIUM_CONVERSION_MAX_TEMP: f32 = 340.0;
pub const PN_TRITIUM_CONVERSION_ENERGY: f32 = 10_000.0;
pub const PN_BZASE_MIN_TEMP: f32 = 260.0;
pub const PN_BZASE_MAX_TEMP: f32 = 280.0;
pub const PN_BZASE_ENERGY: f32 = 60_000.0;
pub const ANTINOBLIUM_CONVERSION_DIVISOR: f32 = 90.0;
pub const REACTION_OPPRESSION_MIN_TEMP: f32 = 20.0;
