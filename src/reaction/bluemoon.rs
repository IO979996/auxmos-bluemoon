//! BlueMoon-Station: реакции атмосферы, перенесённые из DM в Rust.
//! Константы соответствуют code/__DEFINES/reactions.dm и code/__DEFINES/atmospherics.dm.

use crate::gas::{
	constants::*, gas_fusion_power, gas_idx_from_string, with_gas_info, with_mix, with_mix_mut,
	FireProductInfo, GasIDX,
};
use byondapi::prelude::*;
use eyre::Result;

const SUPER_SATURATION_THRESHOLD: f32 = 96.0; // SUPER_SATURATION_THRESHOLD из reactions.dm

/// Вызов из Rust для начисления очков исследований (в DM опционально /proc/bluemoon_add_research_points(amount)).
fn research_add_points(amount: f32) {
	// Прок может отсутствовать — не паникуем
	let _ = byondapi::global_call::call_global_id(
		byond_string!("bluemoon_add_research_points"),
		&[amount.into()],
	)
	.ok();
}

#[must_use]
pub fn func_from_id(id: &str) -> Option<super::ReactFunc> {
	match id {
		"nobstop" => Some(noblium_suppression),
		"vapor" => Some(water_vapor),
		"plasmafire" => Some(plasma_fire),
		"tritfire" => Some(tritium_fire),
		"fusion" => Some(fusion),
		"genericfire" => Some(generic_fire),
		"nitrylformation" => Some(nitryl_formation),
		"bzformation" => Some(bz_formation),
		"stimformation" => Some(stimulum_formation),
		"nobformation" => Some(noblium_formation),
		"sterilization" => Some(miasma_sterilization),
		"nitric_oxide" => Some(nitric_oxide_decomp),
		"hagedorn" => Some(hagedorn),
		"dehagedorn" => Some(dehagedorn),
		"freonfire" => Some(freon_fire),
		"freonformation" => Some(freon_formation),
		"halon_o2removal" => Some(halon_o2_removal),
		"healium_formation" => Some(healium_formation),
		"zauker_formation" => Some(zauker_formation),
		"zauker_decomp" => Some(zauker_decomp),
		"nitrium_formation" => Some(nitrium_formation),
		"nitrium_decomp" => Some(nitrium_decomp),
		"pluox_formation" => Some(pluox_formation),
		"proto_nitrate_formation" => Some(proto_nitrate_formation),
		"proto_nitrate_hydrogen_response" => Some(proto_nitrate_hydrogen_response),
		"proto_nitrate_tritium_response" => Some(proto_nitrate_tritium_response),
		"proto_nitrate_bz_response" => Some(proto_nitrate_bz_response),
		"antinoblium_replication" => Some(antinoblium_replication),
		_ => None,
	}
}

fn noblium_suppression(_byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	// STOP_REACTIONS = 2
	Ok(2.0.into())
}

fn water_vapor(byond_air: ByondValue, holder: ByondValue) -> Result<ByondValue> {
	const WATER_VAPOR_FREEZE: f32 = 200.0;
	let h2o = gas_idx_from_string(GAS_H2O)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp <= WATER_VAPOR_FREEZE {
			let _ = holder.call_id(byond_string!("freon_gas_act"), &[]);
			Ok(false)
		} else {
			let _ = holder.call_id(byond_string!("water_vapor_gas_act"), &[]);
			let moles = air.get_moles(h2o);
			if moles >= MOLES_GAS_VISIBLE {
				air.adjust_moles(h2o, -MOLES_GAS_VISIBLE);
				Ok(true)
			} else {
				Ok(false)
			}
		}
	})?;
	Ok(reacted.into())
}

fn plasma_fire(byond_air: ByondValue, holder: ByondValue) -> Result<ByondValue> {
	// BlueMoon/TG: PLASMA_UPPER_TEMPERATURE = 1370+T0C (atmospherics.dm)
	const PLASMA_UPPER_TEMPERATURE: f32 = 1370.0 + T0C;
	const OXYGEN_BURN_RATE_BASE: f32 = 1.4; // OXYGEN_BURN_RATE_BASE
	const PLASMA_OXYGEN_FULLBURN: f32 = 10.0;
	const PLASMA_BURN_RATE_DELTA: f32 = 9.0; // PLASMA_BURN_RATE_DELTA
	const FIRE_PLASMA_ENERGY_RELEASED: f32 = 3_000_000.0; // FIRE_PLASMA_ENERGY_RELEASED
	let o2 = gas_idx_from_string(GAS_O2)?;
	let plasma = gas_idx_from_string(GAS_PLASMA)?;
	let co2 = gas_idx_from_string(GAS_CO2)?;
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let (oxygen_burn_rate, plasma_burn_rate, initial_oxy, initial_plasma, initial_energy) =
		with_mix(&byond_air, |air| {
			let temperature_scale = {
				if air.get_temperature() > PLASMA_UPPER_TEMPERATURE {
					1.0
				} else {
					(air.get_temperature() - FIRE_MINIMUM_TEMPERATURE_TO_EXIST)
						/ (PLASMA_UPPER_TEMPERATURE - FIRE_MINIMUM_TEMPERATURE_TO_EXIST)
				}
			};
			if temperature_scale > 0.0 {
				let oxygen_burn_rate = OXYGEN_BURN_RATE_BASE - temperature_scale;
				let oxy = air.get_moles(o2);
				let plas = air.get_moles(plasma);
				let plasma_burn_rate = {
					if oxy > plas * PLASMA_OXYGEN_FULLBURN {
						plas * temperature_scale / PLASMA_BURN_RATE_DELTA
					} else {
						(temperature_scale * (oxy / PLASMA_OXYGEN_FULLBURN))
							/ PLASMA_BURN_RATE_DELTA
					}
				}
				.min(plas)
				.min(oxy / oxygen_burn_rate);
				Ok((
					oxygen_burn_rate,
					plasma_burn_rate,
					oxy,
					plas,
					air.thermal_energy(),
				))
			} else {
				Ok((0.0, -1.0, 0.0, 0.0, 0.0))
			}
		})?;
	let fire_amount = plasma_burn_rate * (1.0 + oxygen_burn_rate);
	if fire_amount > 0.0 {
		let temperature = with_mix_mut(&byond_air, |air| {
			air.set_moles(plasma, initial_plasma - plasma_burn_rate);
			air.set_moles(o2, initial_oxy - (plasma_burn_rate * oxygen_burn_rate));
			if initial_oxy / initial_plasma > SUPER_SATURATION_THRESHOLD {
				air.adjust_moles(tritium, plasma_burn_rate);
			} else {
				air.adjust_moles(co2, plasma_burn_rate);
			}
			let new_temp = (initial_energy + plasma_burn_rate * FIRE_PLASMA_ENERGY_RELEASED)
				/ air.heat_capacity();
			air.set_temperature(new_temp);
			air.garbage_collect();
			Ok(new_temp)
		})?;
		let mut cached_results = byond_air.read_var_id(byond_string!("reaction_results"))?;
		cached_results.write_list_index("fire", fire_amount)?;
		if temperature > FIRE_MINIMUM_TEMPERATURE_TO_EXIST {
			byondapi::global_call::call_global_id(
				byond_string!("fire_expose"),
				&[holder, byond_air, temperature.into()],
			)?;
		}
		Ok(true.into())
	} else {
		Ok(false.into())
	}
}

fn tritium_fire(byond_air: ByondValue, holder: ByondValue) -> Result<ByondValue> {
	const TRITIUM_BURN_OXY_FACTOR: f32 = 100.0; // TRITIUM_BURN_OXY_FACTOR
	const TRITIUM_BURN_TRIT_FACTOR: f32 = 10.0; // TRITIUM_BURN_TRIT_FACTOR
	const TRITIUM_MINIMUM_RADIATION_FACTOR: f32 = 0.1; // TRITIUM_MINIMUM_RADIATION_ENERGY
	const FIRE_HYDROGEN_ENERGY_RELEASED: f32 = 280_000.0; // FIRE_HYDROGEN_ENERGY_RELEASED
	let o2 = gas_idx_from_string(GAS_O2)?;
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let water = gas_idx_from_string(GAS_H2O)?;
	let (burned_fuel, energy_released, temperature) = with_mix_mut(&byond_air, |air| {
		let initial_oxy = air.get_moles(o2);
		let initial_trit = air.get_moles(tritium);
		let initial_energy = air.thermal_energy();
		let burned_fuel = {
			if initial_oxy < initial_trit {
				let r = initial_oxy / TRITIUM_BURN_OXY_FACTOR;
				air.set_moles(tritium, initial_trit - r);
				r
			} else {
				let r = initial_trit * TRITIUM_BURN_TRIT_FACTOR;
				air.set_moles(
					tritium,
					initial_trit - initial_trit / TRITIUM_BURN_TRIT_FACTOR,
				);
				air.set_moles(o2, initial_oxy - initial_trit);
				r
			}
		};
		air.adjust_moles(water, burned_fuel / TRITIUM_BURN_OXY_FACTOR);
		let energy_released = FIRE_HYDROGEN_ENERGY_RELEASED * burned_fuel;
		let new_temp = (initial_energy + energy_released) / air.heat_capacity();
		let mut cached_results = byond_air.read_var_id(byond_string!("reaction_results"))?;
		cached_results.write_list_index("fire", burned_fuel)?;
		air.set_temperature(new_temp);
		air.garbage_collect();
		Ok((burned_fuel, energy_released, new_temp))
	})?;
	if burned_fuel > TRITIUM_MINIMUM_RADIATION_FACTOR {
		byondapi::global_call::call_global_id(
			byond_string!("radiation_burn"),
			&[holder, energy_released.into()],
		)?;
	}
	if temperature > FIRE_MINIMUM_TEMPERATURE_TO_EXIST {
		byondapi::global_call::call_global_id(
			byond_string!("fire_expose"),
			&[holder, byond_air, temperature.into()],
		)?;
	}
	Ok(true.into())
}

fn fusion(byond_air: ByondValue, holder: ByondValue) -> Result<ByondValue> {
	// Параметры синтеза (адаптированы под BlueMoon/TG fusion из reactions.dm)
	const TOROID_CALCULATED_THRESHOLD: f32 = 5.96;
	const INSTABILITY_GAS_POWER_FACTOR: f32 = 0.003; // INSTABILITY_GAS_POWER_FACTOR
	const PLASMA_BINDING_ENERGY: f32 = 20_000_000.0; // PLASMA_BINDING_ENERGY
	const FUSION_TRITIUM_MOLES_USED: f32 = 1.0; // FUSION_TRITIUM_MOLES_USED
	const FUSION_INSTABILITY_ENDOTHERMALITY: f32 = 2.0; // FUSION_INSTABILITY_ENDOTHERMALITY
	const FUSION_TRITIUM_CONVERSION_COEFFICIENT: f32 = 1e-10; // FUSION_TRITIUM_CONVERSION_COEFFICIENT (DM); waste = reaction_energy * this
	const FUSION_MOLE_THRESHOLD: f32 = 250.0; // FUSION_MOLE_THRESHOLD
	const FUSION_SCALE_DIVISOR: f32 = 10.0;
	const FUSION_MINIMAL_SCALE: f32 = 50.0;
	const FUSION_SLOPE_DIVISOR: f32 = 1250.0;
	const FUSION_ENERGY_TRANSLATION_EXPONENT: f32 = 1.25;
	const FUSION_BASE_TEMPSCALE: f32 = 6.0;
	const FUSION_MIDDLE_ENERGY_REFERENCE: f32 = 1E+6;
	const FUSION_BUFFER_DIVISOR: f32 = 1.0;
	const INFINITY: f32 = 1E+30;
	let plas = gas_idx_from_string(GAS_PLASMA)?;
	let co2 = gas_idx_from_string(GAS_CO2)?;
	let trit = gas_idx_from_string(GAS_TRITIUM)?;
	let nitrous = gas_idx_from_string(GAS_NITROUS)?;
	let bz = gas_idx_from_string(GAS_BZ)?;
	let nitryl = gas_idx_from_string(GAS_NITRYL)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let (
		initial_energy,
		initial_plasma,
		initial_carbon,
		scale_factor,
		temperature_scale,
		gas_power,
	) = with_mix(&byond_air, |air| {
		Ok((
			air.thermal_energy(),
			air.get_moles(plas),
			air.get_moles(co2),
			(air.volume / FUSION_SCALE_DIVISOR).max(FUSION_MINIMAL_SCALE),
			air.get_temperature().log10(),
			air.enumerate()
				.fold(0.0, |acc, (i, amt)| acc + gas_fusion_power(&i) * amt),
		))
	})?;
	let toroidal_size = TOROID_CALCULATED_THRESHOLD + {
		if temperature_scale <= FUSION_BASE_TEMPSCALE {
			(temperature_scale - FUSION_BASE_TEMPSCALE) / FUSION_BUFFER_DIVISOR
		} else {
			(4.0_f32.powf(temperature_scale - FUSION_BASE_TEMPSCALE)) / FUSION_SLOPE_DIVISOR
		}
	};
	let instability = (gas_power * INSTABILITY_GAS_POWER_FACTOR).rem_euclid(toroidal_size);
	byond_air.call_id(byond_string!("set_analyzer_results"), &[instability.into()])?;
	let mut thermal_energy = initial_energy;

	let mut plasma = (initial_plasma - FUSION_MOLE_THRESHOLD) / scale_factor;
	let mut carbon = (initial_carbon - FUSION_MOLE_THRESHOLD) / scale_factor;

	plasma = (plasma - instability * carbon.sin()).rem_euclid(toroidal_size);
	carbon = (carbon - plasma).rem_euclid(toroidal_size);

	plasma = plasma * scale_factor + FUSION_MOLE_THRESHOLD;
	carbon = carbon * scale_factor + FUSION_MOLE_THRESHOLD;

	let delta_plasma = (initial_plasma - plasma).min(toroidal_size * scale_factor * 1.5);

	let reaction_energy = {
		if (delta_plasma > 0.0) || (instability <= FUSION_INSTABILITY_ENDOTHERMALITY) {
			(delta_plasma * PLASMA_BINDING_ENERGY).max(0.0)
		} else {
			delta_plasma
				* PLASMA_BINDING_ENERGY
				* ((instability - FUSION_INSTABILITY_ENDOTHERMALITY).sqrt())
		}
	};

	if reaction_energy != 0.0 {
		let middle_energy = (((TOROID_CALCULATED_THRESHOLD / 2.0) * scale_factor)
			+ FUSION_MOLE_THRESHOLD)
			* (200.0 * FUSION_MIDDLE_ENERGY_REFERENCE);
		thermal_energy = middle_energy
			* FUSION_ENERGY_TRANSLATION_EXPONENT.powf((thermal_energy / middle_energy).log10());
		let bowdlerized_reaction_energy = reaction_energy.clamp(
			thermal_energy * ((1.0 / (FUSION_ENERGY_TRANSLATION_EXPONENT.powi(2))) - 1.0),
			thermal_energy * (FUSION_ENERGY_TRANSLATION_EXPONENT.powi(2) - 1.0),
		);
		thermal_energy = middle_energy
			* 10_f32.powf(
				((thermal_energy + bowdlerized_reaction_energy) / middle_energy)
					.log(FUSION_ENERGY_TRANSLATION_EXPONENT),
			);
	}

	// DM: waste = FUSION_TRITIUM_MOLES_USED * (reaction_energy * FUSION_TRITIUM_CONVERSION_COEFFICIENT)
	let waste_moles = FUSION_TRITIUM_MOLES_USED
		* (reaction_energy.abs() * FUSION_TRITIUM_CONVERSION_COEFFICIENT)
		.max(scale_factor * FUSION_TRITIUM_CONVERSION_COEFFICIENT * FUSION_TRITIUM_MOLES_USED);

	let standard_energy = with_mix_mut(&byond_air, |air| {
		air.set_moles(plas, plasma);
		air.set_moles(co2, carbon);
		air.adjust_moles(trit, -FUSION_TRITIUM_MOLES_USED);
		if reaction_energy > 0.0 {
			air.adjust_moles(o2, waste_moles);
			air.adjust_moles(nitrous, waste_moles);
		} else {
			air.adjust_moles(bz, waste_moles);
			air.adjust_moles(nitryl, waste_moles);
		}

		let new_heat_cap = air.heat_capacity();
		let standard_energy = 400_f32 * air.get_moles(plas) * air.get_temperature();

		if new_heat_cap > MINIMUM_HEAT_CAPACITY
			&& (reaction_energy != 0.0 || instability <= FUSION_INSTABILITY_ENDOTHERMALITY)
		{
			air.set_temperature((thermal_energy / new_heat_cap).clamp(TCMB, INFINITY));
		}

		air.garbage_collect();
		Ok(standard_energy)
	})?;
	if reaction_energy != 0.0 {
		byondapi::global_call::call_global_id(
			byond_string!("fusion_ball"),
			&[holder, reaction_energy.into(), standard_energy.into()],
		)?;
		Ok(true.into())
	} else if reaction_energy == 0.0 && instability <= FUSION_INSTABILITY_ENDOTHERMALITY {
		Ok(true.into())
	} else {
		Ok(false.into())
	}
}

fn generic_fire(byond_air: ByondValue, holder: ByondValue) -> Result<ByondValue> {
	use hashbrown::HashMap;
	use rustc_hash::FxBuildHasher;
	let mut burn_results: HashMap<GasIDX, f32, FxBuildHasher> = HashMap::with_capacity_and_hasher(
		super::total_num_gases() as usize,
		FxBuildHasher::default(),
	);
	let mut radiation_released = 0.0;
	with_gas_info(|gas_info| {
		if let Some(fire_amount) = with_mix(&byond_air, |air| {
			let (mut fuels, mut oxidizers) = air.get_fire_info_with_lock(gas_info);
			let oxidation_power = oxidizers
				.iter()
				.copied()
				.fold(0.0, |acc, (_, _, power)| acc + power);
			let total_fuel = fuels
				.iter()
				.copied()
				.fold(0.0, |acc, (_, _, power)| acc + power);
			if oxidation_power < GAS_MIN_MOLES {
				Err(eyre::eyre!(
					"Gas has no oxidizer even though it passed oxidizer check!"
				))
			} else if total_fuel <= GAS_MIN_MOLES {
				Err(eyre::eyre!(
					"Gas has no fuel even though it passed fuel check!"
				))
			} else {
				let oxidation_ratio = oxidation_power / total_fuel;
				if oxidation_ratio > 1.0 {
					for (_, amt, power) in &mut oxidizers {
						*amt /= oxidation_ratio;
						*power /= oxidation_ratio;
					}
				} else {
					for (_, amt, power) in &mut fuels {
						*amt *= oxidation_ratio;
						*power *= oxidation_ratio;
					}
				}
				for (i, a, _) in oxidizers.iter().copied().chain(fuels.iter().copied()) {
					let amt = FIRE_MAXIMUM_BURN_RATE * a;
					let this_gas_info = &gas_info[i];
					radiation_released += amt * this_gas_info.fire_radiation_released;
					if let Some(product_info) = this_gas_info.fire_products.as_ref() {
						match product_info {
							FireProductInfo::Generic(products) => {
								for (product_idx, product_amt) in products.iter() {
									burn_results
										.entry(product_idx.get()?)
										.and_modify(|r| *r += product_amt * amt)
										.or_insert_with(|| product_amt * amt);
								}
							}
							FireProductInfo::Plasma => {
								let product = if oxidation_ratio > SUPER_SATURATION_THRESHOLD {
									GAS_TRITIUM
								} else {
									GAS_CO2
								};
								burn_results
									.entry(gas_idx_from_string(product)?)
									.and_modify(|r| *r += amt)
									.or_insert_with(|| amt);
							}
						}
					}
					burn_results
						.entry(i)
						.and_modify(|r| *r -= amt)
						.or_insert(-amt);
				}
				Ok(Some(
					oxidation_power.min(total_fuel) * 2.0 * FIRE_MAXIMUM_BURN_RATE,
				))
			}
		})? {
			let temperature = with_mix_mut(&byond_air, |air| {
				let initial_enthalpy = air.get_temperature()
					* (air.heat_capacity() + R_IDEAL_GAS_EQUATION * air.total_moles());
				let mut delta_enthalpy = 0.0;
				for (&i, &amt) in &burn_results {
					air.adjust_moles(i, amt);
					delta_enthalpy -= amt * gas_info[i].enthalpy;
				}
				air.set_temperature(
					(initial_enthalpy + delta_enthalpy)
						/ (air.heat_capacity() + R_IDEAL_GAS_EQUATION * air.total_moles()),
				);
				Ok(air.get_temperature())
			})?;
			let mut cached_results = byond_air.read_var_id(byond_string!("reaction_results"))?;
			cached_results.write_list_index("fire", fire_amount)?;
			if temperature > FIRE_MINIMUM_TEMPERATURE_TO_EXIST {
				byondapi::global_call::call_global_id(
					byond_string!("fire_expose"),
					&[holder, byond_air, temperature.into()],
				)?;
			}
			if radiation_released > 0.0 {
				byondapi::global_call::call_global_id(
					byond_string!("radiation_burn"),
					&[holder, radiation_released.into()],
				)?;
			}
			Ok((fire_amount > 0.0).into())
		} else {
			Ok(false.into())
		}
	})
}

fn nitryl_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let o2 = gas_idx_from_string(GAS_O2)?;
	let n2 = gas_idx_from_string(GAS_N2)?;
	let nitryl = gas_idx_from_string(GAS_NITRYL)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let heat_eff = (temp / (FIRE_MINIMUM_TEMPERATURE_TO_EXIST * 100.0))
			.min(air.get_moles(o2))
			.min(air.get_moles(n2));
		if heat_eff <= 0.0
			|| air.get_moles(o2) < heat_eff
			|| air.get_moles(n2) < heat_eff
		{
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(o2, -heat_eff);
		air.adjust_moles(n2, -heat_eff);
		air.adjust_moles(nitryl, heat_eff * 2.0);
		let energy_used = heat_eff * NITRYL_FORMATION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap - energy_used) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn bz_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let n2o = gas_idx_from_string(GAS_NITROUS)?;
	let plasma = gas_idx_from_string(GAS_PLASMA)?;
	let bz = gas_idx_from_string(GAS_BZ)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let pressure = air.return_pressure();
		let n2o_moles = air.get_moles(n2o);
		let plasma_moles = air.get_moles(plasma);
		let ratio = (plasma_moles / n2o_moles).max(1.0);
		let eff = (1.0 / ((pressure / (0.1 * ONE_ATMOSPHERE)) * ratio))
			.min(n2o_moles)
			.min(plasma_moles / 2.0);
		if eff <= 0.0 || n2o_moles < eff || plasma_moles < 2.0 * eff {
			return Ok(false);
		}
		let energy = 2.0 * eff * FIRE_CARBON_ENERGY_RELEASED;
		if energy <= 0.0 {
			return Ok(false);
		}
		air.adjust_moles(bz, eff);
		if (eff - n2o_moles).abs() < 0.001 {
			air.adjust_moles(bz, -pressure.min(1.0));
			air.adjust_moles(o2, pressure.min(1.0));
		}
		air.adjust_moles(n2o, -eff);
		air.adjust_moles(plasma, -2.0 * eff);
		research_add_points((eff * eff * BZ_RESEARCH_SCALE).min(BZ_RESEARCH_MAX_AMOUNT));
		let old_cap = air.heat_capacity();
		if old_cap > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn stimulum_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let plasma = gas_idx_from_string(GAS_PLASMA)?;
	let nitryl = gas_idx_from_string(GAS_NITRYL)?;
	let stim = gas_idx_from_string(GAS_STIMULUM)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let heat_scale = (temp / STIMULUM_HEAT_SCALE)
			.min(air.get_moles(tritium))
			.min(air.get_moles(plasma))
			.min(air.get_moles(nitryl));
		if heat_scale <= 0.0
			|| air.get_moles(tritium) < heat_scale
			|| air.get_moles(plasma) < heat_scale
			|| air.get_moles(nitryl) < heat_scale
		{
			return Ok(false);
		}
		let h2 = heat_scale * heat_scale;
		let h3 = h2 * heat_scale;
		let h4 = h3 * heat_scale;
		let h5 = h4 * heat_scale;
		let stim_energy = heat_scale
			+ STIMULUM_FIRST_RISE * h2
			- STIMULUM_FIRST_DROP * h3
			+ STIMULUM_SECOND_RISE * h4
			- STIMULUM_ABSOLUTE_DROP * h5;
		air.adjust_moles(stim, heat_scale / 10.0);
		air.adjust_moles(tritium, -heat_scale);
		air.adjust_moles(plasma, -heat_scale);
		air.adjust_moles(nitryl, -heat_scale);
		research_add_points(STIMULUM_RESEARCH_AMOUNT * stim_energy.max(0.0));
		if stim_energy != 0.0 && air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			let old_cap = air.heat_capacity();
			air.set_temperature(
				((air.get_temperature() * old_cap + stim_energy) / air.heat_capacity())
					.max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn noblium_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let n2 = gas_idx_from_string(GAS_N2)?;
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let bz = gas_idx_from_string(GAS_BZ)?;
	let hypernob = gas_idx_from_string(GAS_HYPERNOB)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > NOBLIUM_FORMATION_MAX_TEMP {
			return Ok(false);
		}
		let n2_moles = air.get_moles(n2);
		let trit_moles = air.get_moles(tritium);
		let bz_moles = air.get_moles(bz);
		let trit_per_nob = 5.0 * trit_moles / (trit_moles + 1000.0 * bz_moles).max(0.001);
		let nob_formed =
			(n2_moles / 10.0).min(trit_moles / trit_per_nob.max(0.005));
		if nob_formed <= 0.0 {
			return Ok(false);
		}
		let trit_consumed = nob_formed * trit_per_nob;
		if trit_consumed > trit_moles || nob_formed * 10.0 > n2_moles {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(n2, -nob_formed * 10.0);
		air.adjust_moles(tritium, -trit_consumed);
		air.adjust_moles(hypernob, nob_formed);
		let energy = nob_formed * NOBLIUM_FORMATION_ENERGY / (bz_moles * 10.0).max(1.0);
		research_add_points(nob_formed * NOBLIUM_RESEARCH_AMOUNT);
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap + energy) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn miasma_sterilization(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let miasma = gas_idx_from_string(GAS_MIASMA)?;
	let h2o = gas_idx_from_string(GAS_H2O)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		if air.get_moles(h2o) > 0.1 {
			return Ok(false);
		}
		let temp = air.get_temperature();
		let cleaned = air.get_moles(miasma).min(
			20.0 + (temp - (T0C + 170.0)) / 20.0,
		);
		if cleaned <= 0.0 {
			return Ok(false);
		}
		air.adjust_moles(miasma, -cleaned);
		air.adjust_moles(o2, cleaned);
		air.set_temperature(temp + cleaned * 0.002);
		research_add_points(cleaned * MIASMA_RESEARCH_AMOUNT);
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn nitric_oxide_decomp(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let nitric = gas_idx_from_string(GAS_NITRIC)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let n2 = gas_idx_from_string(GAS_N2)?;
	let nitryl = gas_idx_from_string(GAS_NITRYL)?;
	with_gas_info(|gas_info| {
		with_mix_mut(&byond_air, |air| {
			let nitric_moles = air.get_moles(nitric);
			let oxygen_moles = air.get_moles(o2);
			let max_amount = (nitric_moles / 8.0).max(GAS_MIN_MOLES);
			let mut enthalpy = air.get_temperature()
				* (air.heat_capacity() + R_IDEAL_GAS_EQUATION * air.total_moles());
			if oxygen_moles > GAS_MIN_MOLES {
				let reaction_amount = (max_amount.min(oxygen_moles) / 4.0).min(oxygen_moles / 4.0);
				air.adjust_moles(nitric, -reaction_amount * 2.0);
				air.adjust_moles(o2, -reaction_amount);
				air.adjust_moles(nitryl, reaction_amount * 2.0);
				let en_nitric = gas_info[nitric].enthalpy;
				let en_nitryl = gas_info[nitryl].enthalpy;
				enthalpy += reaction_amount * (en_nitryl - en_nitric);
			}
			air.adjust_moles(nitric, -max_amount);
			air.adjust_moles(o2, max_amount * 0.5);
			air.adjust_moles(n2, max_amount * 0.5);
			enthalpy -= max_amount * gas_info[nitric].enthalpy;
			air.set_temperature(
				enthalpy
					/ (air.heat_capacity() + R_IDEAL_GAS_EQUATION * air.total_moles()),
			);
			air.garbage_collect();
			Ok(())
		})
	})?;
	Ok(true.into())
}

fn hagedorn(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let qcd = gas_idx_from_string(GAS_QCD)?;
	with_gas_info(|gas_info| {
		with_mix_mut(&byond_air, |air| {
			if air.get_moles(qcd) > 0.0 {
				return Ok(());
			}
			let initial_energy = air.thermal_energy();
			let temp = air.get_temperature();
			for i in 0..super::total_num_gases() {
				air.set_moles(i, 0.0);
			}
			let sh = gas_info[qcd].specific_heat;
			let amount = initial_energy / (temp * sh);
			air.set_moles(qcd, amount);
			let research_amt = (amount * QCD_RESEARCH_AMOUNT).min(100_000.0);
			research_add_points(research_amt);
			air.garbage_collect();
			Ok(())
		})
	})?;
	Ok(true.into())
}

fn dehagedorn(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let qcd = gas_idx_from_string(GAS_QCD)?;
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let hypernob = gas_idx_from_string(GAS_HYPERNOB)?;
	with_gas_info(|gas_info| {
		with_mix_mut(&byond_air, |air| {
			let initial_energy = air.thermal_energy();
			air.set_moles(qcd, 0.0);
			air.set_temperature(air.get_temperature().min(1.8e12));
			let new_temp = air.get_temperature();
			let gas_indices: Vec<GasIDX> = (0..super::total_num_gases())
				.filter(|&i| i != qcd && i != tritium && i != hypernob)
				.collect();
			if gas_indices.is_empty() {
				return Ok(());
			}
			let mut idx = 0usize;
			let n = gas_indices.len();
			while air.thermal_energy() < initial_energy - 1.0 {
				let g = gas_indices[idx % n];
				idx += 1;
				let sh = gas_info[g].specific_heat;
				let add = (initial_energy - air.thermal_energy())
					.min((initial_energy / (sh * new_temp * 20.0)).max(0.1));
				if add <= 0.0 {
					break;
				}
				air.adjust_moles(g, add);
			}
			air.set_temperature(initial_energy / air.heat_capacity());
			air.garbage_collect();
			Ok(())
		})
	})?;
	Ok(true.into())
}

fn freon_fire(byond_air: ByondValue, holder: ByondValue) -> Result<ByondValue> {
	let o2 = gas_idx_from_string(GAS_O2)?;
	let freon = gas_idx_from_string(GAS_FREON)?;
	let proto_nitrate = gas_idx_from_string(GAS_PROTO_NITRATE)?;
	let co2 = gas_idx_from_string(GAS_CO2)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let mut max_burn_temp = FREON_MAXIMUM_BURN_TEMPERATURE;
		if air.get_moles(proto_nitrate) > GAS_MIN_MOLES {
			max_burn_temp = FREON_CATALYST_MAX_TEMPERATURE;
		}
		if temp > max_burn_temp {
			return Ok(false);
		}
		let temp_scale = if temp < FREON_TERMINAL_TEMPERATURE {
			0.0
		} else if temp < FREON_LOWER_TEMPERATURE {
			0.5
		} else {
			(max_burn_temp - temp) / (max_burn_temp - FREON_TERMINAL_TEMPERATURE)
		};
		if temp_scale <= 0.0 {
			return Ok(false);
		}
		let oxygen_burn_ratio = OXYGEN_BURN_RATIO_BASE - temp_scale;
		let freon_moles = air.get_moles(freon);
		let oxygen_moles = air.get_moles(o2);
		let freon_burn_rate = if oxygen_moles < freon_moles * FREON_OXYGEN_FULLBURN {
			((oxygen_moles / FREON_OXYGEN_FULLBURN) / FREON_BURN_RATE_DELTA) * temp_scale
		} else {
			(freon_moles / FREON_BURN_RATE_DELTA) * temp_scale
		};
		if freon_burn_rate < MINIMUM_HEAT_CAPACITY {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		let burn_rate = freon_burn_rate
			.min(freon_moles)
			.min(oxygen_moles * (1.0 / oxygen_burn_ratio));
		air.adjust_moles(freon, -burn_rate);
		air.adjust_moles(o2, -(burn_rate * oxygen_burn_ratio));
		air.adjust_moles(co2, burn_rate);
		let energy_consumed = FIRE_FREON_ENERGY_CONSUMED * burn_rate;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap - energy_consumed) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	if reacted {
		// Опциональный callback на тайле: hot_ice при 120–160 K; если прока нет — не паникуем
		let _ = holder
			.call_id(byond_string!("bluemoon_freon_hot_ice_check"), &[byond_air])
			.ok();
	}
	Ok(reacted.into())
}

fn freon_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let plasma = gas_idx_from_string(GAS_PLASMA)?;
	let co2 = gas_idx_from_string(GAS_CO2)?;
	let bz = gas_idx_from_string(GAS_BZ)?;
	let freon = gas_idx_from_string(GAS_FREON)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let plasma_moles = air.get_moles(plasma);
		let co2_moles = air.get_moles(co2);
		let bz_moles = air.get_moles(bz);
		let heat_factor = (temp - FREON_FORMATION_MIN_TEMPERATURE) / 100.0;
		let min_mole = (plasma_moles / 0.6)
			.min(co2_moles / 0.3)
			.min(bz_moles / 0.1);
		let reaction_units = (heat_factor * min_mole * 0.05)
			.min(plasma_moles / 0.6)
			.min(co2_moles / 0.3)
			.min(bz_moles / 0.1);
		if reaction_units <= 0.0 {
			return Ok(false);
		}
		air.adjust_moles(plasma, -reaction_units * 0.6);
		air.adjust_moles(co2, -reaction_units * 0.3);
		air.adjust_moles(bz, -reaction_units * 0.1);
		air.adjust_moles(freon, reaction_units * 10.0);
		let old_cap = air.heat_capacity();
		let energy_consumed = FREON_FORMATION_ENERGY_CONSUMED * reaction_units;
		if old_cap > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((air.get_temperature() * old_cap - energy_consumed) / air.heat_capacity())
					.max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn halon_o2_removal(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let halon = gas_idx_from_string(GAS_HALON)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let pluox = gas_idx_from_string(GAS_PLUOXIUM)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let halon_moles = air.get_moles(halon);
		let oxygen_moles = air.get_moles(o2);
		let heat_eff = (temp / HALON_COMBUSTION_TEMPERATURE_SCALE)
			.min(halon_moles)
			.min(oxygen_moles / 20.0);
		if heat_eff <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(halon, -heat_eff);
		air.adjust_moles(o2, -(heat_eff * 20.0));
		air.adjust_moles(pluox, heat_eff * 2.5);
		let energy_used = heat_eff * HALON_COMBUSTION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap - energy_used) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn healium_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let bz = gas_idx_from_string(GAS_BZ)?;
	let freon = gas_idx_from_string(GAS_FREON)?;
	let healium = gas_idx_from_string(GAS_HEALIUM)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > HEALIUM_FORMATION_MAX_TEMP {
			return Ok(false);
		}
		let freon_moles = air.get_moles(freon);
		let bz_moles = air.get_moles(bz);
		let heat_eff = (temp * 0.3)
			.min(freon_moles / 2.75)
			.min(bz_moles / 0.25);
		if heat_eff <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(freon, -heat_eff * 2.75);
		air.adjust_moles(bz, -heat_eff * 0.25);
		air.adjust_moles(healium, heat_eff * 3.0);
		let energy = heat_eff * HEALIUM_FORMATION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn zauker_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let hypernob = gas_idx_from_string(GAS_HYPERNOB)?;
	let nitrium = gas_idx_from_string(GAS_NITRIUM)?;
	let zauker = gas_idx_from_string(GAS_ZAUKER)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > ZAUKER_FORMATION_MAX_TEMPERATURE {
			return Ok(false);
		}
		let hypernob_moles = air.get_moles(hypernob);
		let nitrium_moles = air.get_moles(nitrium);
		let heat_eff = (temp * ZAUKER_FORMATION_TEMPERATURE_SCALE)
			.min(hypernob_moles / 0.01)
			.min(nitrium_moles / 0.5);
		if heat_eff <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(hypernob, -heat_eff * 0.01);
		air.adjust_moles(nitrium, -heat_eff * 0.5);
		air.adjust_moles(zauker, heat_eff * 0.5);
		let energy_used = heat_eff * ZAUKER_FORMATION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap - energy_used) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn zauker_decomp(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let n2 = gas_idx_from_string(GAS_N2)?;
	let zauker = gas_idx_from_string(GAS_ZAUKER)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let n2_moles = air.get_moles(n2);
		let zauker_moles = air.get_moles(zauker);
		let burned = ZAUKER_DECOMPOSITION_MAX_RATE
			.min(n2_moles)
			.min(zauker_moles);
		if burned <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		let temp = air.get_temperature();
		air.adjust_moles(zauker, -burned);
		air.adjust_moles(o2, burned * 0.3);
		air.adjust_moles(n2, burned * 0.7);
		let energy = ZAUKER_DECOMPOSITION_ENERGY * burned;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn nitrium_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let n2 = gas_idx_from_string(GAS_N2)?;
	let bz = gas_idx_from_string(GAS_BZ)?;
	let nitrium = gas_idx_from_string(GAS_NITRIUM)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		let heat_eff = (temp / NITRIUM_FORMATION_TEMP_DIVISOR)
			.min(air.get_moles(tritium))
			.min(air.get_moles(n2))
			.min(air.get_moles(bz) / 0.05);
		if heat_eff <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(tritium, -heat_eff);
		air.adjust_moles(n2, -heat_eff);
		air.adjust_moles(bz, -heat_eff * 0.05);
		air.adjust_moles(nitrium, heat_eff);
		let energy_used = heat_eff * NITRIUM_FORMATION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap - energy_used) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn nitrium_decomp(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let nitrium = gas_idx_from_string(GAS_NITRIUM)?;
	let n2 = gas_idx_from_string(GAS_N2)?;
	let hydrogen = gas_idx_from_string(GAS_HYDROGEN)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > NITRIUM_DECOMPOSITION_MAX_TEMP {
			return Ok(false);
		}
		let nitrium_moles = air.get_moles(nitrium);
		let heat_eff = (temp / NITRIUM_DECOMPOSITION_TEMP_DIVISOR).min(nitrium_moles);
		if heat_eff <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(nitrium, -heat_eff);
		air.adjust_moles(n2, heat_eff);
		air.adjust_moles(hydrogen, heat_eff);
		let energy = heat_eff * NITRIUM_DECOMPOSITION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn pluox_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let co2 = gas_idx_from_string(GAS_CO2)?;
	let o2 = gas_idx_from_string(GAS_O2)?;
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let pluox = gas_idx_from_string(GAS_PLUOXIUM)?;
	let hydrogen = gas_idx_from_string(GAS_HYDROGEN)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > PLUOXIUM_FORMATION_MAX_TEMP {
			return Ok(false);
		}
		let co2_moles = air.get_moles(co2);
		let o2_moles = air.get_moles(o2);
		let trit_moles = air.get_moles(tritium);
		let produced = PLUOXIUM_FORMATION_MAX_RATE
			.min(o2_moles * 0.5)
			.min(co2_moles)
			.min(trit_moles * 100.0); // 1/0.01
		if produced <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(co2, -produced);
		air.adjust_moles(o2, -produced * 2.0);
		air.adjust_moles(tritium, -produced * 0.01);
		air.adjust_moles(pluox, produced);
		air.adjust_moles(hydrogen, produced * 0.01);
		let energy = produced * PLUOXIUM_FORMATION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn proto_nitrate_formation(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let pluox = gas_idx_from_string(GAS_PLUOXIUM)?;
	let hydrogen = gas_idx_from_string(GAS_HYDROGEN)?;
	let proto_nitrate = gas_idx_from_string(GAS_PROTO_NITRATE)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > PN_FORMATION_MAX_TEMPERATURE {
			return Ok(false);
		}
		let pluox_moles = air.get_moles(pluox);
		let h2_moles = air.get_moles(hydrogen);
		let heat_eff = (temp * 0.005)
			.min(pluox_moles / 0.2)
			.min(h2_moles / 2.0);
		if heat_eff <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(hydrogen, -heat_eff * 2.0);
		air.adjust_moles(pluox, -heat_eff * 0.2);
		air.adjust_moles(proto_nitrate, heat_eff * 2.2);
		let energy = heat_eff * PN_FORMATION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn proto_nitrate_hydrogen_response(
	byond_air: ByondValue,
	_holder: ByondValue,
) -> Result<ByondValue> {
	let proto_nitrate = gas_idx_from_string(GAS_PROTO_NITRATE)?;
	let hydrogen = gas_idx_from_string(GAS_HYDROGEN)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let proto_moles = air.get_moles(proto_nitrate);
		let h2_moles = air.get_moles(hydrogen);
		let produced = PN_HYDROGEN_CONVERSION_MAX_RATE
			.min(h2_moles)
			.min(proto_moles);
		if produced <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		let temp = air.get_temperature();
		air.adjust_moles(hydrogen, -produced);
		air.adjust_moles(proto_nitrate, produced * 0.5);
		let energy_used = produced * PN_HYDROGEN_CONVERSION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				((temp * old_cap - energy_used) / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn proto_nitrate_tritium_response(
	byond_air: ByondValue,
	_holder: ByondValue,
) -> Result<ByondValue> {
	let proto_nitrate = gas_idx_from_string(GAS_PROTO_NITRATE)?;
	let tritium = gas_idx_from_string(GAS_TRITIUM)?;
	let hydrogen = gas_idx_from_string(GAS_HYDROGEN)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > PN_TRITIUM_CONVERSION_MAX_TEMP {
			return Ok(false);
		}
		let proto_moles = air.get_moles(proto_nitrate);
		let trit_moles = air.get_moles(tritium);
		let produced = (temp / 34.0 * (trit_moles * proto_moles) / (trit_moles + 10.0 * proto_moles))
			.min(trit_moles)
			.min(proto_moles * 100.0); // 1/0.01
		if produced <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(proto_nitrate, -produced * 0.01);
		air.adjust_moles(tritium, -produced);
		air.adjust_moles(hydrogen, produced);
		let energy = produced * PN_TRITIUM_CONVERSION_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn proto_nitrate_bz_response(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let proto_nitrate = gas_idx_from_string(GAS_PROTO_NITRATE)?;
	let bz = gas_idx_from_string(GAS_BZ)?;
	let n2 = gas_idx_from_string(GAS_N2)?;
	let helium = gas_idx_from_string(GAS_HELIUM)?;
	let plasma = gas_idx_from_string(GAS_PLASMA)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let temp = air.get_temperature();
		if temp > PN_BZASE_MAX_TEMP {
			return Ok(false);
		}
		let proto_moles = air.get_moles(proto_nitrate);
		let bz_moles = air.get_moles(bz);
		let consumed = (temp / 2240.0 * bz_moles * proto_moles / (bz_moles + proto_moles))
			.min(bz_moles)
			.min(proto_moles);
		if consumed <= 0.0 {
			return Ok(false);
		}
		let old_cap = air.heat_capacity();
		air.adjust_moles(bz, -consumed);
		air.adjust_moles(proto_nitrate, -consumed);
		air.adjust_moles(n2, consumed * 0.4);
		air.adjust_moles(helium, consumed * 1.6);
		air.adjust_moles(plasma, consumed * 0.8);
		let energy = consumed * PN_BZASE_ENERGY;
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(((temp * old_cap + energy) / air.heat_capacity()).max(TCMB));
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}

fn antinoblium_replication(byond_air: ByondValue, _holder: ByondValue) -> Result<ByondValue> {
	let antinoblium = gas_idx_from_string(GAS_ANTINOBLIUM)?;
	let reacted = with_mix_mut(&byond_air, |air| {
		let total = air.total_moles();
		let antinob_moles = air.get_moles(antinoblium);
		let total_other = total - antinob_moles;
		if total_other < GAS_MIN_MOLES {
			return Ok(false);
		}
		let rate = (antinob_moles / ANTINOBLIUM_CONVERSION_DIVISOR).min(total_other);
		let old_cap = air.heat_capacity();
		for i in 0..super::total_num_gases() {
			if i == antinoblium {
				continue;
			}
			let m = air.get_moles(i);
			if m > 0.0 {
				air.adjust_moles(i, -rate * (m / total_other));
			}
		}
		air.adjust_moles(antinoblium, rate);
		if air.heat_capacity() > MINIMUM_HEAT_CAPACITY {
			air.set_temperature(
				(air.get_temperature() * old_cap / air.heat_capacity()).max(TCMB),
			);
		}
		air.garbage_collect();
		Ok(true)
	})?;
	Ok(reacted.into())
}
