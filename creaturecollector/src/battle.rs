use crate::audio::Sfx;
use crate::data::{
    Affinity, ArtifactId, CreatureInstance, SpeciesId, SpellGemId,
};
use crate::framebuffer::Framebuffer;
use crate::sprites::AssetBank;
use std::collections::VecDeque;

pub const MAX_CHAIN_STEPS: usize = 14;
pub const MAX_TRIGGERS_PER_UNIT_TURN: u8 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Ally,
    Enemy,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Ally => Side::Enemy,
            Side::Enemy => Side::Ally,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatKind {
    Atk,
    Def,
    Int,
    Spd,
}

impl StatKind {
    pub fn tag(self) -> &'static str {
        match self {
            StatKind::Atk => "+ATK",
            StatKind::Def => "+DEF",
            StatKind::Int => "+INT",
            StatKind::Spd => "+SPD",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BattleUnit {
    pub data: CreatureInstance,
    pub max_hp: i32,
    pub cur_hp: i32,
    pub base_atk: i32,
    pub base_def: i32,
    pub base_int: i32,
    pub base_spd: i32,
    // Combat modifiers (stacks of +20% each, capped at +5 = +100%)
    pub atk_stages: i32,
    pub def_stages: i32,
    pub int_stages: i32,
    pub spd_stages: i32,
    pub shield: i32,
    pub poison: i32,
    pub provoking: bool,
    pub defending_hold_fire: bool, // Halves incoming dmg AND suppresses offensive reactive traits!
    pub echo_chorus_used_this_turn: bool,
    pub trait_fires_this_turn: u8,
    pub gauge: i32,
    pub hit_flash: i32,
    pub last_cmd: BattleCmd,
}

impl BattleUnit {
    pub fn from_instance(inst: CreatureInstance) -> Self {
        let max_hp = inst.max_hp();
        let cur_hp = inst.cur_hp.clamp(1, max_hp);
        let last_cmd = BattleCmd::from_u8(inst.last_cmd);
        Self {
            base_atk: inst.atk(),
            base_def: inst.def(),
            base_int: inst.int_stat(),
            base_spd: inst.spd(),
            data: inst,
            max_hp,
            cur_hp,
            atk_stages: 0,
            def_stages: 0,
            int_stages: 0,
            spd_stages: 0,
            shield: 0,
            poison: 0,
            provoking: false,
            defending_hold_fire: false,
            echo_chorus_used_this_turn: false,
            trait_fires_this_turn: 0,
            gauge: 0,
            hit_flash: 0,
            last_cmd,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.cur_hp > 0
    }

    pub fn eff_atk(&self) -> i32 {
        ((self.base_atk * (100 + self.atk_stages * 20)) / 100).max(1)
    }

    pub fn eff_def(&self) -> i32 {
        ((self.base_def * (100 + self.def_stages * 20)) / 100).max(1)
    }

    pub fn eff_int(&self) -> i32 {
        ((self.base_int * (100 + self.int_stages * 20)) / 100).max(1)
    }

    pub fn eff_spd(&self) -> i32 {
        ((self.base_spd * (100 + self.spd_stages * 20)) / 100).max(1)
    }

    pub fn hp_pct(&self) -> i32 {
        if self.max_hp <= 0 {
            0
        } else {
            (self.cur_hp * 100) / self.max_hp
        }
    }

    /// Deterministic capture threshold: 1%..=40% Max HP (non-boss wild creatures only)
    pub fn is_capture_ready(&self) -> bool {
        self.is_alive()
            && !self.data.species.meta().is_boss
            && self.cur_hp * 100 <= self.max_hp * 40
    }

    pub fn capture_hp_cutoff(&self) -> i32 {
        ((self.max_hp * 40) / 100).max(1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BattleCmd {
    Attack = 0,
    CastGem = 1,
    DefendHold = 2,
    Provoke = 3,
    Capture = 4,
}

impl BattleCmd {
    pub const ALL: [BattleCmd; 5] = [
        BattleCmd::Attack,
        BattleCmd::CastGem,
        BattleCmd::DefendHold,
        BattleCmd::Provoke,
        BattleCmd::Capture,
    ];

    pub fn from_u8(v: u8) -> Self {
        Self::ALL.get(v as usize).copied().unwrap_or(BattleCmd::Attack)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BattlePhase {
    BattleIntro { ticks: i32 },
    AdvancingTimeline,
    WaitingAllyCommand { slot: usize },
    WaitingEnemyTarget { ally_slot: usize, cmd: BattleCmd },
    PodCaptureAnim {
        ally_slot: usize,
        enemy_slot: usize,
        ticks: i32,
    },
    ResolvingChain { step_delay: i32 },
    Victory { ticks: i32 },
    Defeat { ticks: i32 },
}

#[derive(Clone, Debug)]
pub enum ChainStep {
    BasicAttack {
        attacker: (Side, usize),
        target: (Side, usize),
        non_lethal_wild: bool,
    },
    CastSpell {
        caster: (Side, usize),
        gem: SpellGemId,
        primary_target: Option<(Side, usize)>,
        is_free_echo: bool,
    },
    TraitHeal {
        source: (Side, usize),
        target: (Side, usize),
        amount: i32,
        label: &'static str,
    },
    TraitDamage {
        source: (Side, usize),
        target: (Side, usize),
        raw_power: i32,
        affinity: Affinity,
        add_poison: i32,
        is_spell_like: bool,
        non_lethal_wild: bool,
        label: &'static str,
    },
    TraitBuff {
        source: (Side, usize),
        target: (Side, usize),
        stat: StatKind,
        label: &'static str,
    },
    TraitShield {
        source: (Side, usize),
        target: (Side, usize),
        amount: i32,
        label: &'static str,
    },
}

#[derive(Clone)]
pub struct VisualArc {
    pub from: (Side, usize),
    pub to: (Side, usize),
    pub col: u8,
    pub ticks: i32,
}

#[derive(Clone)]
pub struct PopBadge {
    pub side: Side,
    pub slot: usize,
    pub text: String,
    pub col: u8,
    pub ticks: i32,
}

pub struct BattleState {
    pub allies: [Option<BattleUnit>; 6],
    pub enemies: [Option<BattleUnit>; 6],
    pub is_boss_battle: bool,
    pub zone_num: usize,
    pub banner_title: String,

    pub phase: BattlePhase,
    pub cmd_cursor: usize,
    pub target_cursor: usize,
    pub turbo_auto: bool,

    pub pending_steps: VecDeque<ChainStep>,
    pub chain_depth: usize,
    pub log_line_1: String,
    pub log_line_2: String,

    pub arcs: Vec<VisualArc>,
    pub pops: Vec<PopBadge>,
    pub captured_units: Vec<CreatureInstance>,
    pub rng_state: u32,
    pub xp_reward: i32,
}

pub fn is_front_col(slot: usize) -> bool {
    slot < 3
}

pub fn row_of(slot: usize) -> usize {
    slot % 3
}

pub fn row_mate(slot: usize) -> usize {
    if slot < 3 {
        slot + 3
    } else {
        slot - 3
    }
}

pub fn adjacent_slots(slot: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(3);
    let r = row_of(slot);
    let col_base = if slot < 3 { 0 } else { 3 };
    if r > 0 {
        out.push(col_base + r - 1);
    }
    if r < 2 {
        out.push(col_base + r + 1);
    }
    out.push(row_mate(slot));
    out
}

/// Exact `84 x 50` card coordinates on the `400 x 300` VGA screen (`y = 33..187`)
pub fn card_rect(side: Side, slot: usize) -> (i32, i32, i32, i32) {
    let r = (slot % 3) as i32;
    let y = 33 + r * 52;
    let x = match (side, slot < 3) {
        (Side::Ally, false) => 8,   // Ally Back Column (3..6)
        (Side::Ally, true) => 98,   // Ally Front Column (0..3)
        (Side::Enemy, true) => 218, // Enemy Front Column (0..3)
        (Side::Enemy, false) => 308,// Enemy Back Column (3..6)
    };
    (x, y, 84, 50)
}

pub fn card_center(side: Side, slot: usize) -> (i32, i32) {
    let (x, y, _w, _h) = card_rect(side, slot);
    // Anchor arcs directly on the 24x24 monster sprite center (x + 17, y + 25) so numbers stay unobstructed!
    (x + 17, y + 25)
}

impl BattleState {
    pub fn new(
        ally_squad: &[Option<CreatureInstance>; 6],
        enemy_squad: [Option<CreatureInstance>; 6],
        is_boss_battle: bool,
        zone_num: usize,
        banner_title: String,
        seed: u32,
    ) -> Self {
        let mut allies: [Option<BattleUnit>; 6] = [None, None, None, None, None, None];
        let mut enemies: [Option<BattleUnit>; 6] = [None, None, None, None, None, None];
        let mut xp_reward = 0;

        for i in 0..6 {
            if let Some(ref c) = ally_squad[i] {
                if c.cur_hp > 0 {
                    let mut u = BattleUnit::from_instance(c.clone());
                    u.gauge = (u.eff_spd() * 14).min(650);
                    allies[i] = Some(u);
                }
            }
            if let Some(ref e) = enemy_squad[i] {
                xp_reward += 10 + e.level * 5 + if e.is_alpha { 12 } else { 0 };
                let mut u = BattleUnit::from_instance(e.clone());
                u.gauge = (u.eff_spd() * 12).min(600);
                enemies[i] = Some(u);
            }
        }
        if is_boss_battle {
            xp_reward += 28;
        }

        let mut bs = Self {
            allies,
            enemies,
            is_boss_battle,
            zone_num,
            banner_title: banner_title.clone(),
            phase: BattlePhase::BattleIntro { ticks: 8 },
            cmd_cursor: 0,
            target_cursor: 0,
            turbo_auto: false,
            pending_steps: VecDeque::new(),
            chain_depth: 0,
            log_line_1: format!("ENCOUNTER: {}", banner_title),
            log_line_2: "TIMELINE ENGAGED - BUILD TRAIT CHAINS!".to_string(),
            arcs: Vec::new(),
            pops: Vec::new(),
            captured_units: Vec::new(),
            rng_state: seed.max(1),
            xp_reward,
        };
        bs.enqueue_battle_start_traits();
        bs
    }

    fn next_rand(&mut self) -> u32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x.max(1);
        self.rng_state
    }

    pub fn push_log(&mut self, msg: String) {
        self.log_line_1 = std::mem::replace(&mut self.log_line_2, msg);
    }

    pub fn unit(&self, side: Side, slot: usize) -> Option<&BattleUnit> {
        let arr = match side {
            Side::Ally => &self.allies,
            Side::Enemy => &self.enemies,
        };
        arr.get(slot).and_then(|u| u.as_ref()).filter(|u| u.is_alive())
    }

    pub fn unit_mut(&mut self, side: Side, slot: usize) -> Option<&mut BattleUnit> {
        let arr = match side {
            Side::Ally => &mut self.allies,
            Side::Enemy => &mut self.enemies,
        };
        arr.get_mut(slot)
            .and_then(|u| u.as_mut())
            .filter(|u| u.is_alive())
    }

    pub fn living_slots(&self, side: Side) -> Vec<usize> {
        (0..6).filter(|&s| self.unit(side, s).is_some()).collect()
    }

    fn can_fire_trait(&mut self, side: Side, slot: usize, is_offensive: bool) -> bool {
        if self.chain_depth >= MAX_CHAIN_STEPS {
            return false;
        }
        if let Some(u) = self.unit_mut(side, slot) {
            if is_offensive && u.defending_hold_fire {
                return false;
            }
            if u.trait_fires_this_turn >= MAX_TRIGGERS_PER_UNIT_TURN {
                return false;
            }
            u.trait_fires_this_turn += 1;
            true
        } else {
            false
        }
    }

    fn reset_per_turn_trait_counters(&mut self) {
        self.chain_depth = 0;
        for s in 0..6 {
            if let Some(Some(u)) = self.allies.get_mut(s) {
                u.trait_fires_this_turn = 0;
                u.echo_chorus_used_this_turn = false;
            }
            if let Some(Some(u)) = self.enemies.get_mut(s) {
                u.trait_fires_this_turn = 0;
                u.echo_chorus_used_this_turn = false;
            }
        }
    }

    fn enqueue_battle_start_traits(&mut self) {
        for side in [Side::Ally, Side::Enemy] {
            for slot in 0..6 {
                let Some((sp, art, max_hp, def)) = self.unit(side, slot).map(|u| {
                    (u.data.species, u.data.artifact, u.max_hp, u.eff_def())
                }) else {
                    continue;
                };
                // Lantern Hermit-Crab: Tidal Bulwark shields Front Row at battle start!
                if sp == SpeciesId::LanternCrab {
                    let shd = (def * 90) / 100 + 6;
                    for fs in 0..3 {
                        if self.unit(side, fs).is_some() {
                            self.pending_steps.push_back(ChainStep::TraitShield {
                                source: (side, slot),
                                target: (side, fs),
                                amount: shd,
                                label: "TIDAL BULWARK",
                            });
                        }
                    }
                }
                if let Some(a) = art {
                    match a {
                        ArtifactId::IronbarkBuckler => {
                            self.pending_steps.push_back(ChainStep::TraitShield {
                                source: (side, slot),
                                target: (side, slot),
                                amount: (max_hp * 35) / 100,
                                label: "IRONBARK BUCKLER",
                            });
                        }
                        ArtifactId::ResonantCore => {
                            self.pending_steps.push_back(ChainStep::TraitBuff {
                                source: (side, slot),
                                target: (side, slot),
                                stat: StatKind::Atk,
                                label: "RESONANT CORE",
                            });
                            self.pending_steps.push_back(ChainStep::TraitBuff {
                                source: (side, slot),
                                target: (side, slot),
                                stat: StatKind::Def,
                                label: "RESONANT CORE",
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Previews upcoming 8 turns on the Top Timeline Bar without mutating state
    pub fn preview_timeline(&self) -> Vec<(Side, usize)> {
        let mut sim: Vec<(Side, usize, i32, i32)> = Vec::new();
        for side in [Side::Ally, Side::Enemy] {
            for slot in 0..6 {
                if let Some(u) = self.unit(side, slot) {
                    sim.push((side, slot, u.gauge, u.eff_spd()));
                }
            }
        }
        if sim.is_empty() {
            return Vec::new();
        }
        let mut order = Vec::with_capacity(8);
        if let BattlePhase::WaitingAllyCommand { slot }
        | BattlePhase::WaitingEnemyTarget {
            ally_slot: slot, ..
        } = self.phase
        {
            if self.unit(Side::Ally, slot).is_some() {
                order.push((Side::Ally, slot));
            }
        }

        for _ in 0..16 {
            if order.len() >= 8 {
                break;
            }
            for item in &mut sim {
                item.2 += item.3;
            }
            sim.sort_by(|a, b| b.2.cmp(&a.2));
            if sim[0].2 >= 100 {
                sim[0].2 -= 100;
                order.push((sim[0].0, sim[0].1));
            }
        }
        order
    }

    fn pick_random_living(&mut self, side: Side) -> Option<usize> {
        let slots = self.living_slots(side);
        if slots.is_empty() {
            None
        } else {
            let idx = (self.next_rand() as usize) % slots.len();
            Some(slots[idx])
        }
    }

    fn pick_default_enemy_target(&self) -> Option<usize> {
        // Respect Provoke first, then Front Row, then Back Row
        for s in 0..6 {
            if let Some(u) = self.unit(Side::Enemy, s) {
                if u.provoking {
                    return Some(s);
                }
            }
        }
        for s in 0..6 {
            if self.unit(Side::Enemy, s).is_some() {
                return Some(s);
            }
        }
        None
    }

    fn pick_capture_priority_target(&self) -> Option<usize> {
        for s in 0..6 {
            if let Some(u) = self.unit(Side::Enemy, s) {
                if u.is_capture_ready() {
                    return Some(s);
                }
            }
        }
        self.pick_default_enemy_target()
    }

    fn pick_lowest_hp_slot(&self, side: Side) -> Option<usize> {
        self.living_slots(side)
            .into_iter()
            .min_by_key(|&s| self.unit(side, s).map(|u| u.hp_pct()).unwrap_or(999))
    }

    fn pick_highest_hp_slot(&self, side: Side) -> Option<usize> {
        self.living_slots(side)
            .into_iter()
            .max_by_key(|&s| self.unit(side, s).map(|u| u.cur_hp).unwrap_or(0))
    }

    fn step_delay_ticks(&self) -> i32 {
        if self.turbo_auto {
            2
        } else {
            6
        }
    }

    pub fn add_pop(&mut self, side: Side, slot: usize, text: String, col: u8) {
        self.pops.retain(|p| !(p.side == side && p.slot == slot));
        self.pops.push(PopBadge {
            side,
            slot,
            text,
            col,
            ticks: if self.turbo_auto { 6 } else { 11 },
        });
    }

    pub fn add_arc(&mut self, from: (Side, usize), to: (Side, usize), col: u8) {
        if from != to {
            self.arcs.push(VisualArc {
                from,
                to,
                col,
                ticks: if self.turbo_auto { 3 } else { 6 },
            });
        }
    }

    /// Advances battle by 1 discrete 15 FPS tick.
    pub fn tick_animations_and_flow(&mut self, pods_left: &mut usize, sfx_out: &mut Vec<Sfx>) {
        for a in &mut self.arcs {
            a.ticks -= 1;
        }
        self.arcs.retain(|a| a.ticks > 0);

        for p in &mut self.pops {
            p.ticks -= 1;
        }
        self.pops.retain(|p| p.ticks > 0);

        for s in 0..6 {
            if let Some(Some(u)) = self.allies.get_mut(s) {
                if u.hit_flash > 0 {
                    u.hit_flash -= 1;
                }
            }
            if let Some(Some(u)) = self.enemies.get_mut(s) {
                if u.hit_flash > 0 {
                    u.hit_flash -= 1;
                }
            }
        }

        // Check end-of-battle conditions cleanly when no animation/chain is mid-flight
        if !matches!(
            self.phase,
            BattlePhase::Victory { .. }
                | BattlePhase::Defeat { .. }
                | BattlePhase::PodCaptureAnim { .. }
        ) {
            if self.living_slots(Side::Enemy).is_empty() {
                self.pending_steps.clear();
                self.push_log("VICTORY! SPECIMEN DATA SECURED!".to_string());
                sfx_out.push(Sfx::CoinStart);
                self.phase = BattlePhase::Victory { ticks: 12 };
                return;
            }
            if self.living_slots(Side::Ally).is_empty() {
                self.pending_steps.clear();
                self.push_log("SQUAD WIPED OUT! EMERGENCY RETREAT...".to_string());
                sfx_out.push(Sfx::Hurt);
                self.phase = BattlePhase::Defeat { ticks: 16 };
                return;
            }
        }

        match self.phase {
            BattlePhase::BattleIntro { ticks } => {
                if ticks <= 1 || self.turbo_auto {
                    if !self.pending_steps.is_empty() {
                        self.phase = BattlePhase::ResolvingChain { step_delay: 1 };
                    } else {
                        self.phase = BattlePhase::AdvancingTimeline;
                    }
                } else {
                    self.phase = BattlePhase::BattleIntro { ticks: ticks - 1 };
                }
            }
            BattlePhase::PodCaptureAnim {
                ally_slot: _,
                enemy_slot,
                ticks,
            } => {
                if ticks <= 1 {
                    if let Some(u) = self.enemies[enemy_slot].take() {
                        let mut captured = u.data;
                        captured.heal_full();
                        let name = captured.species.meta().name;
                        self.push_log(format!(
                            "CAPTURED {} (LV.{})!",
                            name, captured.level
                        ));
                        self.captured_units.push(captured);
                        sfx_out.push(Sfx::ServiceBell);
                    }
                    if self.living_slots(Side::Enemy).is_empty() {
                        self.phase = BattlePhase::Victory { ticks: 10 };
                    } else {
                        self.phase = BattlePhase::AdvancingTimeline;
                    }
                } else {
                    self.phase = BattlePhase::PodCaptureAnim {
                        ally_slot: 0,
                        enemy_slot,
                        ticks: ticks - 1,
                    };
                }
            }
            BattlePhase::ResolvingChain { step_delay } => {
                if step_delay > 1 {
                    self.phase = BattlePhase::ResolvingChain {
                        step_delay: step_delay - 1,
                    };
                } else if let Some(step) = self.pending_steps.pop_front() {
                    self.execute_chain_step(step, sfx_out);
                    if self.pending_steps.is_empty() {
                        self.phase = BattlePhase::AdvancingTimeline;
                    } else {
                        self.phase = BattlePhase::ResolvingChain {
                            step_delay: self.step_delay_ticks(),
                        };
                    }
                } else {
                    self.phase = BattlePhase::AdvancingTimeline;
                }
            }
            BattlePhase::AdvancingTimeline => {
                self.advance_to_next_turn(pods_left, sfx_out);
            }
            BattlePhase::WaitingAllyCommand { slot } => {
                if self.turbo_auto {
                    self.auto_choose_ally_turn(slot, sfx_out);
                }
            }
            BattlePhase::WaitingEnemyTarget { ally_slot, .. } => {
                if self.turbo_auto {
                    self.auto_choose_ally_turn(ally_slot, sfx_out);
                }
            }
            BattlePhase::Victory { ticks } => {
                if ticks > 0 {
                    self.phase = BattlePhase::Victory { ticks: ticks - 1 };
                }
            }
            BattlePhase::Defeat { ticks } => {
                if ticks > 0 {
                    self.phase = BattlePhase::Defeat { ticks: ticks - 1 };
                }
            }
        }
    }

    fn advance_to_next_turn(&mut self, _pods_left: &mut usize, sfx_out: &mut Vec<Sfx>) {
        self.reset_per_turn_trait_counters();

        // Advance timeline gauges until fastest unit hits >= 100
        let mut best: Option<(Side, usize, i32)> = None;
        for _ in 0..30 {
            for s in 0..6 {
                if let Some(u) = self.unit_mut(Side::Ally, s) {
                    u.gauge += u.eff_spd();
                    if u.gauge >= 100
                        && best.map(|b| u.gauge > b.2).unwrap_or(true)
                    {
                        best = Some((Side::Ally, s, u.gauge));
                    }
                }
                if let Some(u) = self.unit_mut(Side::Enemy, s) {
                    u.gauge += u.eff_spd();
                    if u.gauge >= 100
                        && best.map(|b| u.gauge > b.2).unwrap_or(true)
                    {
                        best = Some((Side::Enemy, s, u.gauge));
                    }
                }
            }
            if best.is_some() {
                break;
            }
        }

        let Some((side, slot, _)) = best else {
            return;
        };

        // Start-of-turn upkeep (clear Defend/Provoke stance, apply Poison & Turn-Start traits)
        let mut poison_dmg = 0;
        let mut turn_sp = SpeciesId::DewNewt;
        let mut turn_art = None;
        let mut max_hp = 1;
        if let Some(u) = self.unit_mut(side, slot) {
            u.gauge = (u.gauge - 100).max(0);
            u.defending_hold_fire = false;
            u.provoking = false;
            turn_sp = u.data.species;
            turn_art = u.data.artifact;
            max_hp = u.max_hp;
            if u.poison > 0 {
                poison_dmg = ((u.max_hp * 10 * u.poison.min(3)) / 100).max(2);
                u.poison = (u.poison - 1).max(0);
            }
        }

        if poison_dmg > 0 {
            self.apply_direct_hp_loss(side, slot, poison_dmg, true);
            self.add_pop(side, slot, format!("-{}PSN", poison_dmg), 30);
            sfx_out.push(Sfx::SuctionStick);
            if self.unit(side, slot).is_none() {
                return;
            }
        }

        // Turn-start Artifact & Boss traits!
        if turn_art == Some(ArtifactId::DewdropVial) {
            self.pending_steps.push_back(ChainStep::TraitHeal {
                source: (side, slot),
                target: (side, slot),
                amount: (max_hp * 15) / 100 + 2,
                label: "DEWDROP VIAL",
            });
        }
        if turn_sp == SpeciesId::BossDaot {
            self.pending_steps.push_back(ChainStep::CastSpell {
                caster: (side, slot),
                gem: SpellGemId::ChainArc,
                primary_target: None,
                is_free_echo: true,
            });
        } else if turn_sp == SpeciesId::BossGrof {
            self.pending_steps.push_back(ChainStep::TraitHeal {
                source: (side, slot),
                target: (side, slot),
                amount: (max_hp * 10) / 100 + 4,
                label: "DARK DOMINION",
            });
        }

        if side == Side::Ally {
            if self.turbo_auto {
                self.auto_choose_ally_turn(slot, sfx_out);
            } else {
                self.cmd_cursor = self
                    .unit(Side::Ally, slot)
                    .map(|u| u.last_cmd as usize)
                    .unwrap_or(0);
                self.target_cursor = self.pick_default_enemy_target().unwrap_or(0);
                self.phase = BattlePhase::WaitingAllyCommand { slot };
            }
        } else {
            self.choose_enemy_ai_turn(slot);
            self.phase = BattlePhase::ResolvingChain {
                step_delay: self.step_delay_ticks(),
            };
        }
    }

    fn auto_choose_ally_turn(&mut self, slot: usize, _sfx_out: &mut Vec<Sfx>) {
        let Some((sp, gem, hp_pct)) = self.unit(Side::Ally, slot).map(|u| {
            (u.data.species, u.data.spell_gem, u.hp_pct())
        }) else {
            self.phase = BattlePhase::AdvancingTimeline;
            return;
        };

        let any_ally_hurt = self
            .living_slots(Side::Ally)
            .into_iter()
            .any(|s| self.unit(Side::Ally, s).map(|u| u.hp_pct() < 75).unwrap_or(false));

        // Smart Turbo AI: Casters/healers cast their Spell Gems, heavy tanks provoke when healthy, strikers attack!
        let use_provoke = matches!(
            sp,
            SpeciesId::BriarToad | SpeciesId::LanternCrab | SpeciesId::BogGolem
        ) && hp_pct > 60
            && (self.next_rand() % 3 == 0);

        let use_cast = match gem {
            SpellGemId::MistPulse | SpellGemId::StarlightChoir => any_ally_hurt,
            SpellGemId::SporeVolley
            | SpellGemId::ChainArc
            | SpellGemId::AncientNova
            | SpellGemId::SiphonHex
            | SpellGemId::BrambleWard
            | SpellGemId::MonolithSlam => true,
            SpellGemId::VenomLance | SpellGemId::PrecisionDart => self.next_rand() % 2 == 0,
        };

        if use_provoke {
            self.commit_ally_command(slot, BattleCmd::Provoke, None);
        } else if use_cast {
            let tgt = self.pick_default_enemy_target();
            self.commit_ally_command(slot, BattleCmd::CastGem, tgt);
        } else if let Some(tgt) = self.pick_default_enemy_target() {
            self.commit_ally_command(slot, BattleCmd::Attack, Some(tgt));
        }
    }

    fn choose_enemy_ai_turn(&mut self, slot: usize) {
        let Some((sp, gem, int_val, atk_val)) = self.unit(Side::Enemy, slot).map(|u| {
            (u.data.species, u.data.spell_gem, u.eff_int(), u.eff_atk())
        }) else {
            return;
        };

        // Pick an ally target (respecting Ally Provoke & Front Row preference!)
        let mut target = None;
        for s in 0..6 {
            if let Some(u) = self.unit(Side::Ally, s) {
                if u.provoking {
                    target = Some(s);
                    break;
                }
            }
        }
        if target.is_none() {
            let front: Vec<usize> = (0..3)
                .filter(|&s| self.unit(Side::Ally, s).is_some())
                .collect();
            if !front.is_empty() {
                target = Some(front[(self.next_rand() as usize) % front.len()]);
            } else {
                target = self.pick_random_living(Side::Ally);
            }
        }

        let cast_roll = self.next_rand() % 100;
        let wants_cast = matches!(sp, SpeciesId::BossDaot | SpeciesId::BossGrof)
            || (int_val >= atk_val && cast_roll < 55);

        if wants_cast {
            self.pending_steps.push_back(ChainStep::CastSpell {
                caster: (Side::Enemy, slot),
                gem,
                primary_target: target.map(|t| (Side::Ally, t)),
                is_free_echo: false,
            });
        } else if let Some(t) = target {
            self.pending_steps.push_back(ChainStep::BasicAttack {
                attacker: (Side::Enemy, slot),
                target: (Side::Ally, t),
                non_lethal_wild: false,
            });
        }
        self.enqueue_post_action_traits(Side::Enemy, slot, false);
    }

    pub fn commit_ally_command(
        &mut self,
        ally_slot: usize,
        cmd: BattleCmd,
        enemy_target: Option<usize>,
    ) {
        if let Some(u) = self.unit_mut(Side::Ally, ally_slot) {
            u.last_cmd = cmd;
            if cmd != BattleCmd::Capture {
                u.data.last_cmd = cmd as u8;
            }
        }
        let Some((sp, gem, art)) = self.unit(Side::Ally, ally_slot).map(|u| {
            (u.data.species, u.data.spell_gem, u.data.artifact)
        }) else {
            return;
        };
        let ally_name = sp.meta().short_name;

        match cmd {
            BattleCmd::Attack => {
                if let Some(eslot) = enemy_target {
                    let non_lethal = art == Some(ArtifactId::TrapperLens) && !self.is_boss_battle;
                    self.pending_steps.push_back(ChainStep::BasicAttack {
                        attacker: (Side::Ally, ally_slot),
                        target: (Side::Enemy, eslot),
                        non_lethal_wild: non_lethal,
                    });
                    // Crown Salamander: Astral Follow-Up auto-casts Spell Gem after Basic Attack!
                    if sp == SpeciesId::CrownSalamander {
                        self.pending_steps.push_back(ChainStep::CastSpell {
                            caster: (Side::Ally, ally_slot),
                            gem,
                            primary_target: Some((Side::Enemy, eslot)),
                            is_free_echo: true,
                        });
                    }
                    self.enqueue_post_action_traits(Side::Ally, ally_slot, false);
                }
            }
            BattleCmd::CastGem => {
                self.pending_steps.push_back(ChainStep::CastSpell {
                    caster: (Side::Ally, ally_slot),
                    gem,
                    primary_target: enemy_target.map(|e| (Side::Enemy, e)),
                    is_free_echo: false,
                });
                self.enqueue_post_action_traits(Side::Ally, ally_slot, false);
            }
            BattleCmd::DefendHold => {
                if let Some(u) = self.unit_mut(Side::Ally, ally_slot) {
                    u.defending_hold_fire = true;
                }
                self.push_log(format!(
                    "{} DEFENDS (50% DMG & HOLDS OFFENSIVE CHAINS)!",
                    ally_name
                ));
                self.add_pop(Side::Ally, ally_slot, "HOLD!".to_string(), 34);
                self.enqueue_on_defend_traits(Side::Ally, ally_slot);
                self.enqueue_post_action_traits(Side::Ally, ally_slot, true);
            }
            BattleCmd::Provoke => {
                if let Some(u) = self.unit_mut(Side::Ally, ally_slot) {
                    u.provoking = true;
                }
                self.push_log(format!("{} PROVOKES ENEMY FORMATION!", ally_name));
                self.add_pop(Side::Ally, ally_slot, "TAUNT!".to_string(), 15);
                self.enqueue_on_provoke_traits(Side::Ally, ally_slot);
                self.enqueue_post_action_traits(Side::Ally, ally_slot, false);
            }
            BattleCmd::Capture => {}
        }

        self.phase = BattlePhase::ResolvingChain {
            step_delay: if self.pending_steps.is_empty() {
                1
            } else {
                self.step_delay_ticks()
            },
        };
    }

    fn enqueue_post_action_traits(&mut self, side: Side, slot: usize, _was_defend: bool) {
        let Some((sp, _max_hp)) = self
            .unit(side, slot)
            .map(|u| (u.data.species, u.max_hp))
        else {
            return;
        };
        // Twen Dew-Newt: Soothing Dew heals orthogonally adjacent allies after acting or defending!
        if sp == SpeciesId::DewNewt && self.can_fire_trait(side, slot, false) {
            for adj in adjacent_slots(slot) {
                if let Some(au) = self.unit(side, adj) {
                    let amt = ((au.max_hp * 22) / 100).max(4);
                    self.pending_steps.push_back(ChainStep::TraitHeal {
                        source: (side, slot),
                        target: (side, adj),
                        amount: amt,
                        label: "SOOTHING DEW",
                    });
                }
            }
        }
    }

    fn enqueue_on_defend_traits(&mut self, side: Side, slot: usize) {
        let Some((sp, art, max_hp)) = self.unit(side, slot).map(|u| {
            (u.data.species, u.data.artifact, u.max_hp)
        }) else {
            return;
        };

        if sp == SpeciesId::BriarToad && self.can_fire_trait(side, slot, false) {
            self.pending_steps.push_back(ChainStep::TraitHeal {
                source: (side, slot),
                target: (side, slot),
                amount: (max_hp * 18) / 100 + 3,
                label: "BRAMBLE HIDE",
            });
        } else if sp == SpeciesId::StarCarp && self.can_fire_trait(side, slot, false) {
            let mate = row_mate(slot);
            let target_slot = if self.unit(side, mate).is_some() {
                mate
            } else {
                slot
            };
            self.pending_steps.push_back(ChainStep::TraitShield {
                source: (side, slot),
                target: (side, target_slot),
                amount: (max_hp * 25) / 100 + 4,
                label: "ASTRAL CURRENT",
            });
            self.pending_steps.push_back(ChainStep::TraitBuff {
                source: (side, slot),
                target: (side, target_slot),
                stat: StatKind::Spd,
                label: "ASTRAL CURRENT",
            });
        } else if sp == SpeciesId::BogGolem && self.can_fire_trait(side, slot, false) {
            for fs in 0..3 {
                if self.unit(side, fs).is_some() {
                    self.pending_steps.push_back(ChainStep::TraitBuff {
                        source: (side, slot),
                        target: (side, fs),
                        stat: StatKind::Def,
                        label: "MONOLITH AEGIS",
                    });
                }
            }
        }

        if art == Some(ArtifactId::ChorusChime) {
            let mate = row_mate(slot);
            let target_slot = if self.unit(side, mate).is_some() {
                mate
            } else {
                slot
            };
            if let Some(tu) = self.unit(side, target_slot) {
                self.pending_steps.push_back(ChainStep::TraitHeal {
                    source: (side, slot),
                    target: (side, target_slot),
                    amount: (tu.max_hp * 25) / 100 + 3,
                    label: "CHORUS CHIME",
                });
            }
        }
    }

    fn enqueue_on_provoke_traits(&mut self, side: Side, slot: usize) {
        let Some((sp, art, gem, max_hp, def)) = self.unit(side, slot).map(|u| {
            (
                u.data.species,
                u.data.artifact,
                u.data.spell_gem,
                u.max_hp,
                u.eff_def(),
            )
        }) else {
            return;
        };

        match sp {
            SpeciesId::BriarToad => {
                if self.can_fire_trait(side, slot, false) {
                    self.pending_steps.push_back(ChainStep::TraitHeal {
                        source: (side, slot),
                        target: (side, slot),
                        amount: (max_hp * 16) / 100 + 3,
                        label: "BRAMBLE HIDE",
                    });
                    if let Some(foe) = self.pick_random_living(side.opposite()) {
                        self.pending_steps.push_back(ChainStep::TraitDamage {
                            source: (side, slot),
                            target: (side.opposite(), foe),
                            raw_power: (def * 55) / 100 + 3,
                            affinity: Affinity::Verdant,
                            add_poison: 0,
                            is_spell_like: false,
                            non_lethal_wild: false,
                            label: "BRAMBLE THORNS",
                        });
                    }
                }
            }
            SpeciesId::LanternCrab => {
                if self.can_fire_trait(side, slot, false) {
                    for fs in 0..3 {
                        if self.unit(side, fs).is_some() {
                            self.pending_steps.push_back(ChainStep::TraitShield {
                                source: (side, slot),
                                target: (side, fs),
                                amount: (def * 80) / 100 + 5,
                                label: "TIDAL BULWARK",
                            });
                        }
                    }
                }
            }
            SpeciesId::BogGolem => {
                if self.can_fire_trait(side, slot, false) {
                    for fs in 0..3 {
                        if self.unit(side, fs).is_some() {
                            self.pending_steps.push_back(ChainStep::TraitBuff {
                                source: (side, slot),
                                target: (side, fs),
                                stat: StatKind::Def,
                                label: "MONOLITH AEGIS",
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        if art == Some(ArtifactId::LanternPrism) {
            self.pending_steps.push_back(ChainStep::CastSpell {
                caster: (side, slot),
                gem,
                primary_target: None,
                is_free_echo: true,
            });
        }
    }

    fn execute_chain_step(&mut self, step: ChainStep, sfx_out: &mut Vec<Sfx>) {
        self.chain_depth += 1;
        match step {
            ChainStep::BasicAttack {
                attacker,
                target,
                non_lethal_wild,
            } => {
                let Some((atk_sp, atk_val, atk_aff, atk_art)) =
                    self.unit(attacker.0, attacker.1).map(|u| {
                        (
                            u.data.species,
                            u.eff_atk(),
                            u.data.species.meta().affinity,
                            u.data.artifact,
                        )
                    })
                else {
                    return;
                };

                // Redirect target if dead, or Bog-Idol Golem guarding Back Row!
                let mut actual_tgt = target;
                if self.unit(actual_tgt.0, actual_tgt.1).is_none() {
                    if let Some(fallback) = self.pick_random_living(actual_tgt.0) {
                        actual_tgt.1 = fallback;
                    } else {
                        return;
                    }
                }

                let mut guard_split = false;
                if !is_front_col(actual_tgt.1) {
                    let front_guard = row_mate(actual_tgt.1);
                    if let Some(gu) = self.unit(actual_tgt.0, front_guard) {
                        if gu.data.species == SpeciesId::BogGolem {
                            actual_tgt.1 = front_guard;
                            guard_split = true;
                        }
                    }
                }

                let mut bonus_mult = 100;
                if let Some(tu) = self.unit(actual_tgt.0, actual_tgt.1) {
                    if tu.poison > 0 && atk_art == Some(ArtifactId::AssassinFang) {
                        bonus_mult += 35;
                    }
                }

                let raw = (atk_val * bonus_mult) / 100;
                let mut dealt = self.deal_mitigated_damage(
                    attacker,
                    actual_tgt,
                    raw,
                    atk_aff,
                    false,
                    non_lethal_wild,
                );
                if guard_split {
                    dealt = (dealt * 3) / 4;
                }

                // Venom-Lash Viper: Detonate target Poison stacks on basic attack!
                if atk_sp == SpeciesId::VenomViper {
                    let psn = self
                        .unit(actual_tgt.0, actual_tgt.1)
                        .map(|u| u.poison)
                        .unwrap_or(0);
                    if psn > 0 {
                        let burst = psn * (atk_val / 2 + 4);
                        if let Some(tu) = self.unit_mut(actual_tgt.0, actual_tgt.1) {
                            tu.poison = 0;
                        }
                        dealt += self.apply_direct_hp_loss_clamped(
                            actual_tgt.0,
                            actual_tgt.1,
                            burst,
                            non_lethal_wild && actual_tgt.0 == Side::Enemy,
                        );
                    }
                }

                let atk_name = atk_sp.meta().short_name;
                let tgt_name = self
                    .enemies
                    .get(actual_tgt.1)
                    .and_then(|o| o.as_ref())
                    .or_else(|| self.allies.get(actual_tgt.1).and_then(|o| o.as_ref()))
                    .map(|u| u.data.species.meta().short_name)
                    .unwrap_or("FOE");

                self.add_arc(attacker, actual_tgt, 14);
                self.push_log(format!("{} STRIKES {} FOR {} DMG!", atk_name, tgt_name, dealt));
                sfx_out.push(Sfx::CleaverChop);
                self.check_on_hit_and_death_reactions(attacker, actual_tgt, false);
            }

            ChainStep::CastSpell {
                caster,
                gem,
                primary_target,
                is_free_echo,
            } => {
                let Some((c_sp, c_int, c_def, c_spd, c_aff, c_art, c_max_hp)) =
                    self.unit(caster.0, caster.1).map(|u| {
                        (
                            u.data.species,
                            u.eff_int(),
                            u.eff_def(),
                            u.eff_spd(),
                            u.data.species.meta().affinity,
                            u.data.artifact,
                            u.max_hp,
                        )
                    })
                else {
                    return;
                };

                let foe_side = caster.0.opposite();
                let prefix = if is_free_echo { "[ECHO] " } else { "" };
                self.push_log(format!(
                    "{}{} CASTS {}!",
                    prefix,
                    c_sp.meta().short_name,
                    gem.meta().short_name
                ));
                sfx_out.push(Sfx::ElectricZap);

                match gem {
                    SpellGemId::MistPulse => {
                        let amt = (c_int * 65) / 100 + 6;
                        let mut targets = vec![caster.1];
                        targets.extend(adjacent_slots(caster.1));
                        for ts in targets {
                            if self.unit(caster.0, ts).is_some() {
                                self.apply_heal(caster, (caster.0, ts), amt);
                            }
                        }
                    }
                    SpellGemId::SporeVolley => {
                        let foes = self.living_slots(foe_side);
                        for &fs in foes.iter().take(2) {
                            let raw = (c_int * 80) / 100 + 3;
                            self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                Affinity::Verdant,
                                true,
                                false,
                            );
                            self.inflict_poison(caster, (foe_side, fs), 2);
                            self.add_arc(caster, (foe_side, fs), 25);
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                    }
                    SpellGemId::ChainArc => {
                        let foes = self.living_slots(foe_side);
                        for &fs in foes.iter().take(3) {
                            let raw = (c_int * 68) / 100 + 3;
                            self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                Affinity::Tide,
                                true,
                                false,
                            );
                            self.add_arc(caster, (foe_side, fs), 34);
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                    }
                    SpellGemId::BrambleWard => {
                        let col_start = if caster.1 < 3 { 0 } else { 3 };
                        let shd = (c_max_hp * 28) / 100 + 5;
                        for s in col_start..(col_start + 3) {
                            if self.unit(caster.0, s).is_some() {
                                self.apply_shield(caster, (caster.0, s), shd);
                                self.apply_stat_buff(caster, (caster.0, s), StatKind::Def);
                            }
                        }
                    }
                    SpellGemId::VenomLance => {
                        let t_slot = primary_target
                            .filter(|t| self.unit(t.0, t.1).is_some())
                            .map(|t| t.1)
                            .or_else(|| self.pick_random_living(foe_side));
                        if let Some(fs) = t_slot {
                            let raw = (c_int * 115) / 100 + 4;
                            self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                Affinity::Verdant,
                                true,
                                false,
                            );
                            self.inflict_poison(caster, (foe_side, fs), 2);
                            self.add_arc(caster, (foe_side, fs), 26);
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                    }
                    SpellGemId::StarlightChoir => {
                        let amt = (c_int * 48) / 100 + 5;
                        for s in self.living_slots(caster.0) {
                            self.apply_heal(caster, (caster.0, s), amt);
                            self.apply_stat_buff(caster, (caster.0, s), StatKind::Spd);
                        }
                    }
                    SpellGemId::MonolithSlam => {
                        let mut front_foes: Vec<usize> = (0..3)
                            .filter(|&s| self.unit(foe_side, s).is_some())
                            .collect();
                        if front_foes.is_empty() {
                            front_foes = self.living_slots(foe_side);
                        }
                        for fs in front_foes {
                            let raw = (c_def * 105) / 100 + 4;
                            self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                Affinity::Construct,
                                true,
                                false,
                            );
                            self.add_arc(caster, (foe_side, fs), 41);
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                    }
                    SpellGemId::SiphonHex => {
                        let t_slot = primary_target
                            .filter(|t| self.unit(t.0, t.1).is_some())
                            .map(|t| t.1)
                            .or_else(|| self.pick_random_living(foe_side));
                        if let Some(fs) = t_slot {
                            let raw = (c_int * 105) / 100 + 4;
                            let dealt = self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                Affinity::Arcane,
                                true,
                                false,
                            );
                            self.add_arc(caster, (foe_side, fs), 30);
                            if let Some(low) = self.pick_lowest_hp_slot(caster.0) {
                                self.apply_heal(caster, (caster.0, low), (dealt * 65) / 100 + 4);
                            }
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                    }
                    SpellGemId::PrecisionDart => {
                        let t_slot = primary_target
                            .filter(|t| self.unit(t.0, t.1).is_some())
                            .map(|t| t.1)
                            .or_else(|| self.pick_random_living(foe_side));
                        if let Some(fs) = t_slot {
                            let raw = (c_spd * 110) / 100 + 5;
                            let non_lethal = !self.is_boss_battle && foe_side == Side::Enemy;
                            self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                c_aff,
                                true,
                                non_lethal,
                            );
                            self.add_arc(caster, (foe_side, fs), 15);
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                    }
                    SpellGemId::AncientNova => {
                        for fs in self.living_slots(foe_side) {
                            let raw = (c_int * 72) / 100 + 3;
                            self.deal_mitigated_damage(
                                caster,
                                (foe_side, fs),
                                raw,
                                Affinity::Arcane,
                                true,
                                false,
                            );
                            self.add_arc(caster, (foe_side, fs), 30);
                            self.check_on_hit_and_death_reactions(caster, (foe_side, fs), true);
                        }
                        self.apply_stat_buff(caster, caster, StatKind::Int);
                    }
                }

                // On-Cast Self/Artifact/Party/Enemy-Boss Triggers!
                if c_sp == SpeciesId::StarCarp && self.can_fire_trait(caster.0, caster.1, false) {
                    let mate = row_mate(caster.1);
                    let ts = if self.unit(caster.0, mate).is_some() {
                        mate
                    } else {
                        caster.1
                    };
                    self.pending_steps.push_back(ChainStep::TraitShield {
                        source: caster,
                        target: (caster.0, ts),
                        amount: (c_max_hp * 24) / 100 + 3,
                        label: "ASTRAL CURRENT",
                    });
                    self.pending_steps.push_back(ChainStep::TraitBuff {
                        source: caster,
                        target: (caster.0, ts),
                        stat: StatKind::Spd,
                        label: "ASTRAL CURRENT",
                    });
                }

                if c_art == Some(ArtifactId::CopperDynamo) {
                    self.pending_steps.push_back(ChainStep::TraitBuff {
                        source: caster,
                        target: caster,
                        stat: StatKind::Spd,
                        label: "COPPER DYNAMO",
                    });
                } else if c_art == Some(ArtifactId::SporeCrown) {
                    if let Some(rf) = self.pick_random_living(foe_side) {
                        self.inflict_poison(caster, (foe_side, rf), 2);
                    }
                }

                // Voltaic Reed-Eel: Arc Conduit whenever ANY ally casts a Spell!
                for s in self.living_slots(caster.0) {
                    let is_eel = self
                        .unit(caster.0, s)
                        .map(|u| u.data.species == SpeciesId::VoltaicEel)
                        .unwrap_or(false);
                    if is_eel && self.can_fire_trait(caster.0, s, true) {
                        let eel_spd = self.unit(caster.0, s).map(|u| u.eff_spd()).unwrap_or(10);
                        for &fs in self.living_slots(foe_side).iter().take(2) {
                            self.pending_steps.push_back(ChainStep::TraitDamage {
                                source: (caster.0, s),
                                target: (foe_side, fs),
                                raw_power: (eel_spd * 45) / 100 + 3,
                                affinity: Affinity::Tide,
                                add_poison: 0,
                                is_spell_like: true,
                                non_lethal_wild: false,
                                label: "ARC CONDUIT",
                            });
                        }
                    }
                }

                // Dew-Weaver Frog: Echo Chorus (1st time/turn an adjacent ally casts, auto-cast own Gem!)
                if !is_free_echo {
                    for adj in adjacent_slots(caster.1) {
                        let frog_gem = self.unit_mut(caster.0, adj).and_then(|u| {
                            if u.data.species == SpeciesId::DewFrog
                                && !u.echo_chorus_used_this_turn
                                && !u.defending_hold_fire
                            {
                                u.echo_chorus_used_this_turn = true;
                                Some(u.data.spell_gem)
                            } else {
                                None
                            }
                        });
                        if let Some(fg) = frog_gem {
                            if self.chain_depth < MAX_CHAIN_STEPS {
                                self.pending_steps.push_back(ChainStep::CastSpell {
                                    caster: (caster.0, adj),
                                    gem: fg,
                                    primary_target: None,
                                    is_free_echo: true,
                                });
                            }
                        }
                    }
                }

                // Boss Grof: Dark Dominion counters enemy spellcasts!
                for fs in self.living_slots(foe_side) {
                    let is_grof = self
                        .unit(foe_side, fs)
                        .map(|u| u.data.species == SpeciesId::BossGrof)
                        .unwrap_or(false);
                    if is_grof && self.can_fire_trait(foe_side, fs, true) {
                        let g_int = self.unit(foe_side, fs).map(|u| u.eff_int()).unwrap_or(18);
                        self.pending_steps.push_back(ChainStep::TraitBuff {
                            source: (foe_side, fs),
                            target: (foe_side, fs),
                            stat: StatKind::Int,
                            label: "DARK DOMINION",
                        });
                        self.pending_steps.push_back(ChainStep::TraitDamage {
                            source: (foe_side, fs),
                            target: caster,
                            raw_power: (g_int * 55) / 100 + 4,
                            affinity: Affinity::Arcane,
                            add_poison: 1,
                            is_spell_like: true,
                            non_lethal_wild: false,
                            label: "HEX LANCE",
                        });
                    }
                }
            }

            ChainStep::TraitHeal {
                source,
                target,
                amount,
                label,
            } => {
                if self.unit(target.0, target.1).is_some() {
                    let healed = self.apply_heal(source, target, amount);
                    if healed > 0 {
                        let tname = self
                            .unit(target.0, target.1)
                            .map(|u| u.data.species.meta().short_name)
                            .unwrap_or("ALLY");
                        self.push_log(format!(
                            "> [{}] HEALS {} +{} HP!",
                            label, tname, healed
                        ));
                        sfx_out.push(Sfx::HarvestPluck);
                    }
                }
            }

            ChainStep::TraitDamage {
                source,
                target,
                raw_power,
                affinity,
                add_poison,
                is_spell_like,
                non_lethal_wild,
                label,
            } => {
                if self.unit(source.0, source.1).is_none() {
                    return;
                }
                let mut actual_tgt = target;
                if self.unit(actual_tgt.0, actual_tgt.1).is_none() {
                    if let Some(fb) = self.pick_random_living(actual_tgt.0) {
                        actual_tgt.1 = fb;
                    } else {
                        return;
                    }
                }
                let dealt = self.deal_mitigated_damage(
                    source,
                    actual_tgt,
                    raw_power,
                    affinity,
                    is_spell_like,
                    non_lethal_wild,
                );
                if add_poison > 0 {
                    self.inflict_poison(source, actual_tgt, add_poison);
                }
                self.add_arc(source, actual_tgt, affinity.color());
                self.push_log(format!("> [{}] DEALS {} DMG!", label, dealt));
                sfx_out.push(Sfx::SuctionLaunch);
                self.check_on_hit_and_death_reactions(source, actual_tgt, is_spell_like);
            }

            ChainStep::TraitBuff {
                source,
                target,
                stat,
                label,
            } => {
                if self.apply_stat_buff(source, target, stat) {
                    let tname = self
                        .unit(target.0, target.1)
                        .map(|u| u.data.species.meta().short_name)
                        .unwrap_or("ALLY");
                    self.push_log(format!("> [{}] {} {}!", label, tname, stat.tag()));
                }
            }

            ChainStep::TraitShield {
                source,
                target,
                amount,
                label,
            } => {
                if self.unit(target.0, target.1).is_some() {
                    self.apply_shield(source, target, amount);
                    let tname = self
                        .unit(target.0, target.1)
                        .map(|u| u.data.species.meta().short_name)
                        .unwrap_or("ALLY");
                    self.push_log(format!("> [{}] SHIELDS {} (+{})!", label, tname, amount));
                }
            }
        }
    }

    fn apply_heal(&mut self, source: (Side, usize), target: (Side, usize), amount: i32) -> i32 {
        let amt = amount.max(1);
        let healed = if let Some(tu) = self.unit_mut(target.0, target.1) {
            let before = tu.cur_hp;
            tu.cur_hp = (tu.cur_hp + amt).min(tu.max_hp);
            (tu.cur_hp - before).max(1) // Still triggers On-Heal traits even at full HP!
        } else {
            return 0;
        };
        self.add_arc(source, target, 25);
        self.add_pop(target.0, target.1, format!("+{}", amt), 25);

        // Spore-Cap Morel: Mycelial Bloom triggers whenever an ally is Healed!
        let foe_side = target.0.opposite();
        for s in self.living_slots(target.0) {
            let is_morel = self
                .unit(target.0, s)
                .map(|u| u.data.species == SpeciesId::SporeMorel)
                .unwrap_or(false);
            if is_morel && self.can_fire_trait(target.0, s, true) {
                let m_int = self.unit(target.0, s).map(|u| u.eff_int()).unwrap_or(12);
                if let Some(rf) = self.pick_random_living(foe_side) {
                    self.pending_steps.push_back(ChainStep::TraitDamage {
                        source: (target.0, s),
                        target: (foe_side, rf),
                        raw_power: (m_int * 45) / 100 + 3,
                        affinity: Affinity::Verdant,
                        add_poison: 1,
                        is_spell_like: true,
                        non_lethal_wild: false,
                        label: "MYCELIAL BLOOM",
                    });
                }
            }
        }
        healed
    }

    fn apply_shield(&mut self, source: (Side, usize), target: (Side, usize), amount: i32) {
        let amt = amount.max(2);
        if let Some(tu) = self.unit_mut(target.0, target.1) {
            tu.shield = (tu.shield + amt).min(tu.max_hp);
        }
        self.add_arc(source, target, 34);
        self.add_pop(target.0, target.1, format!("+{}SHD", amt), 34);

        // Lantern Hermit-Crab: Tidal Bulwark wave splash when an ally gains Shield!
        let foe_side = target.0.opposite();
        for s in self.living_slots(target.0) {
            let is_crab = self
                .unit(target.0, s)
                .map(|u| u.data.species == SpeciesId::LanternCrab)
                .unwrap_or(false);
            if is_crab && self.can_fire_trait(target.0, s, true) {
                let c_def = self.unit(target.0, s).map(|u| u.eff_def()).unwrap_or(14);
                let front_foe = (0..3)
                    .find(|&fs| self.unit(foe_side, fs).is_some())
                    .or_else(|| self.pick_random_living(foe_side));
                if let Some(ff) = front_foe {
                    self.pending_steps.push_back(ChainStep::TraitDamage {
                        source: (target.0, s),
                        target: (foe_side, ff),
                        raw_power: (c_def * 45) / 100 + 3,
                        affinity: Affinity::Tide,
                        add_poison: 0,
                        is_spell_like: false,
                        non_lethal_wild: false,
                        label: "TIDAL WAVE",
                    });
                }
            }
        }
    }

    fn apply_stat_buff(
        &mut self,
        source: (Side, usize),
        target: (Side, usize),
        stat: StatKind,
    ) -> bool {
        let changed = if let Some(tu) = self.unit_mut(target.0, target.1) {
            let stage_ref = match stat {
                StatKind::Atk => &mut tu.atk_stages,
                StatKind::Def => &mut tu.def_stages,
                StatKind::Int => &mut tu.int_stages,
                StatKind::Spd => &mut tu.spd_stages,
            };
            if *stage_ref < 5 {
                *stage_ref += 1;
                true
            } else {
                false
            }
        } else {
            false
        };

        if changed {
            self.add_arc(source, target, 15);
            self.add_pop(target.0, target.1, stat.tag().to_string(), 15);

            // Rune-Carapace Beetle: Resonant Plating copies ally stat buffs & strikes highest-HP foe!
            let foe_side = target.0.opposite();
            for s in self.living_slots(target.0) {
                if s == target.1 {
                    continue;
                }
                let is_beetle = self
                    .unit(target.0, s)
                    .map(|u| u.data.species == SpeciesId::RuneBeetle)
                    .unwrap_or(false);
                if is_beetle && self.can_fire_trait(target.0, s, true) {
                    let b_def = self.unit(target.0, s).map(|u| u.eff_def()).unwrap_or(14);
                    if let Some(bu) = self.unit_mut(target.0, s) {
                        let sr = match stat {
                            StatKind::Atk => &mut bu.atk_stages,
                            StatKind::Def => &mut bu.def_stages,
                            StatKind::Int => &mut bu.int_stages,
                            StatKind::Spd => &mut bu.spd_stages,
                        };
                        *sr = (*sr + 1).min(5);
                    }
                    if let Some(top_foe) = self.pick_highest_hp_slot(foe_side) {
                        self.pending_steps.push_back(ChainStep::TraitDamage {
                            source: (target.0, s),
                            target: (foe_side, top_foe),
                            raw_power: (b_def * 45) / 100 + 3,
                            affinity: Affinity::Construct,
                            add_poison: 0,
                            is_spell_like: false,
                            non_lethal_wild: false,
                            label: "RESONANT PLATING",
                        });
                    }
                }
            }
        }
        changed
    }

    fn inflict_poison(&mut self, _source: (Side, usize), target: (Side, usize), stacks: i32) {
        if let Some(tu) = self.unit_mut(target.0, target.1) {
            tu.poison = (tu.poison + stacks).min(4);
            self.add_pop(target.0, target.1, "PSN!".to_string(), 30);
        } else {
            return;
        }

        // Venom-Lash Viper: Neurotoxin Fang gains +20% ATK whenever an enemy is Poisoned!
        let ally_side = target.0.opposite();
        for s in self.living_slots(ally_side) {
            let is_viper = self
                .unit(ally_side, s)
                .map(|u| u.data.species == SpeciesId::VenomViper)
                .unwrap_or(false);
            if is_viper && self.can_fire_trait(ally_side, s, false) {
                self.pending_steps.push_back(ChainStep::TraitBuff {
                    source: (ally_side, s),
                    target: (ally_side, s),
                    stat: StatKind::Atk,
                    label: "NEUROTOXIN FANG",
                });
            }
        }
    }

    fn deal_mitigated_damage(
        &mut self,
        _attacker: (Side, usize),
        target: (Side, usize),
        raw_power: i32,
        atk_affinity: Affinity,
        _is_spell: bool,
        non_lethal_wild: bool,
    ) -> i32 {
        let Some((t_def, t_aff, t_defending)) = self.unit(target.0, target.1).map(|u| {
            (
                u.eff_def(),
                u.data.species.meta().affinity,
                u.defending_hold_fire,
            )
        }) else {
            return 0;
        };

        let aff_mult = atk_affinity.damage_mult_pct(t_aff);
        // Classic crisp RPG formula: raw * 24 / (18 + def)
        let mut dmg = ((raw_power * 24) / (18 + t_def)).max(2);
        dmg = ((dmg * aff_mult) / 100).max(2);
        if t_defending {
            dmg = (dmg / 2).max(1);
        }

        let dealt = self.apply_direct_hp_loss_clamped(
            target.0,
            target.1,
            dmg,
            non_lethal_wild && target.0 == Side::Enemy,
        );
        let col = if aff_mult > 100 { 15 } else { 38 };
        self.add_pop(target.0, target.1, format!("-{}", dealt), col);
        dealt
    }

    fn apply_direct_hp_loss(&mut self, side: Side, slot: usize, dmg: i32, non_lethal: bool) -> i32 {
        self.apply_direct_hp_loss_clamped(side, slot, dmg, non_lethal)
    }

    fn apply_direct_hp_loss_clamped(
        &mut self,
        side: Side,
        slot: usize,
        mut dmg: i32,
        non_lethal: bool,
    ) -> i32 {
        let Some(u) = self.unit_mut(side, slot) else {
            return 0;
        };
        u.hit_flash = 3;
        let mut total_absorbed_and_hp = 0;
        if u.shield > 0 {
            let shd_hit = dmg.min(u.shield);
            u.shield -= shd_hit;
            dmg -= shd_hit;
            total_absorbed_and_hp += shd_hit;
        }
        if dmg > 0 {
            let floor_hp = if non_lethal { 1 } else { 0 };
            let before = u.cur_hp;
            u.cur_hp = (u.cur_hp - dmg).max(floor_hp);
            total_absorbed_and_hp += before - u.cur_hp;
        }
        total_absorbed_and_hp
    }

    fn check_on_hit_and_death_reactions(
        &mut self,
        attacker: (Side, usize),
        target: (Side, usize),
        was_spell: bool,
    ) {
        let target_alive = self.unit(target.0, target.1).is_some();

        // Briar-Thorn Toad: Bramble Hide thorns when hit while alive!
        if target_alive {
            let toad_def = self.unit(target.0, target.1).and_then(|u| {
                if u.data.species == SpeciesId::BriarToad {
                    Some(u.eff_def())
                } else {
                    None
                }
            });
            if let Some(t_def) = toad_def {
                let hold = self
                    .unit(target.0, target.1)
                    .map(|u| u.defending_hold_fire)
                    .unwrap_or(false);
                if !hold
                    && self.unit(attacker.0, attacker.1).is_some()
                    && self.can_fire_trait(target.0, target.1, true)
                {
                    self.pending_steps.push_back(ChainStep::TraitDamage {
                        source: target,
                        target: attacker,
                        raw_power: (t_def * 55) / 100 + 2,
                        affinity: Affinity::Verdant,
                        add_poison: 0,
                        is_spell_like: false,
                        non_lethal_wild: false,
                        label: "BRAMBLE THORNS",
                    });
                }
            }
        }

        // Gloom-Wisp Moth: Soul Siphon when foe takes Spell dmg OR dies!
        if was_spell || !target_alive {
            let moth_side = target.0.opposite();
            for s in self.living_slots(moth_side) {
                let is_moth = self
                    .unit(moth_side, s)
                    .map(|u| u.data.species == SpeciesId::GloomMoth)
                    .unwrap_or(false);
                if is_moth && self.can_fire_trait(moth_side, s, false) {
                    let mate = row_mate(s);
                    let buff_tgt = if self.unit(moth_side, mate).is_some() {
                        mate
                    } else {
                        s
                    };
                    self.pending_steps.push_back(ChainStep::TraitBuff {
                        source: (moth_side, s),
                        target: (moth_side, buff_tgt),
                        stat: StatKind::Int,
                        label: "SOUL SIPHON",
                    });
                    if let Some(low) = self.pick_lowest_hp_slot(moth_side) {
                        let amt = self
                            .unit(moth_side, low)
                            .map(|u| (u.max_hp * 15) / 100 + 3)
                            .unwrap_or(6);
                        self.pending_steps.push_back(ChainStep::TraitHeal {
                            source: (moth_side, s),
                            target: (moth_side, low),
                            amount: amt,
                            label: "SOUL SIPHON",
                        });
                    }
                }
            }
        }

        // On-Kill triggers (Crown Salamander + Ancient Sigil + Boss Daot ally-fall)
        if !target_alive {
            if let Some((k_sp, k_art, k_max)) = self.unit(attacker.0, attacker.1).map(|u| {
                (u.data.species, u.data.artifact, u.max_hp)
            }) {
                if k_sp == SpeciesId::CrownSalamander
                    && self.can_fire_trait(attacker.0, attacker.1, false)
                {
                    for s in self.living_slots(attacker.0) {
                        self.pending_steps.push_back(ChainStep::TraitBuff {
                            source: attacker,
                            target: (attacker.0, s),
                            stat: StatKind::Atk,
                            label: "CROWN RALLY",
                        });
                    }
                }
                if k_art == Some(ArtifactId::AncientSigil) {
                    self.pending_steps.push_back(ChainStep::TraitHeal {
                        source: attacker,
                        target: attacker,
                        amount: (k_max * 25) / 100 + 4,
                        label: "ANCIENT SIGIL",
                    });
                }
            }

            // Boss Daot shields retinue when one of his allies falls!
            for s in self.living_slots(target.0) {
                let is_daot = self
                    .unit(target.0, s)
                    .map(|u| u.data.species == SpeciesId::BossDaot)
                    .unwrap_or(false);
                if is_daot && self.can_fire_trait(target.0, s, false) {
                    for rs in self.living_slots(target.0) {
                        self.pending_steps.push_back(ChainStep::TraitShield {
                            source: (target.0, s),
                            target: (target.0, rs),
                            amount: 18,
                            label: "ENCHANTER WARD",
                        });
                    }
                }
            }
        }
    }

    pub fn handle_player_input(
        &mut self,
        up_p: bool,
        down_p: bool,
        left_p: bool,
        right_p: bool,
        confirm_p: bool,
        cancel_p: bool,
        turbo_p: bool,
        pods_left: &mut usize,
        sfx_out: &mut Vec<Sfx>,
    ) {
        if turbo_p {
            self.turbo_auto = !self.turbo_auto;
            sfx_out.push(Sfx::SuctionStick);
        }

        match self.phase {
            BattlePhase::WaitingAllyCommand { slot } => {
                if up_p {
                    self.cmd_cursor = (self.cmd_cursor + 4) % 5;
                    sfx_out.push(Sfx::SuctionStick);
                } else if down_p {
                    self.cmd_cursor = (self.cmd_cursor + 1) % 5;
                    sfx_out.push(Sfx::SuctionStick);
                } else if confirm_p {
                    let cmd = BattleCmd::ALL[self.cmd_cursor];
                    match cmd {
                        BattleCmd::Attack => {
                            self.target_cursor = self.pick_default_enemy_target().unwrap_or(0);
                            self.phase = BattlePhase::WaitingEnemyTarget {
                                ally_slot: slot,
                                cmd,
                            };
                            sfx_out.push(Sfx::SuctionStick);
                        }
                        BattleCmd::CastGem => {
                            let needs_tgt = self
                                .unit(Side::Ally, slot)
                                .map(|u| u.data.spell_gem.meta().needs_enemy_target)
                                .unwrap_or(false);
                            if needs_tgt {
                                self.target_cursor = self.pick_default_enemy_target().unwrap_or(0);
                                self.phase = BattlePhase::WaitingEnemyTarget {
                                    ally_slot: slot,
                                    cmd,
                                };
                                sfx_out.push(Sfx::SuctionStick);
                            } else {
                                self.commit_ally_command(slot, BattleCmd::CastGem, None);
                            }
                        }
                        BattleCmd::DefendHold | BattleCmd::Provoke => {
                            self.commit_ally_command(slot, cmd, None);
                        }
                        BattleCmd::Capture => {
                            if self.is_boss_battle {
                                self.push_log(
                                    "BOSS GUARDIANS CANNOT BE CAPTURED!".to_string(),
                                );
                                sfx_out.push(Sfx::Hurt);
                            } else if *pods_left == 0 {
                                self.push_log(
                                    "0 CONTAINMENT PODS LEFT! VISIT MED-BEACON!".to_string(),
                                );
                                sfx_out.push(Sfx::Hurt);
                            } else {
                                self.target_cursor =
                                    self.pick_capture_priority_target().unwrap_or(0);
                                self.phase = BattlePhase::WaitingEnemyTarget {
                                    ally_slot: slot,
                                    cmd: BattleCmd::Capture,
                                };
                                sfx_out.push(Sfx::SuctionStick);
                            }
                        }
                    }
                }
            }

            BattlePhase::WaitingEnemyTarget { ally_slot, cmd } => {
                if cancel_p {
                    self.phase = BattlePhase::WaitingAllyCommand { slot: ally_slot };
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }
                if up_p || down_p || left_p || right_p {
                    self.move_enemy_target_cursor(up_p, down_p, left_p, right_p);
                    sfx_out.push(Sfx::SuctionStick);
                } else if confirm_p {
                    if self.unit(Side::Enemy, self.target_cursor).is_none() {
                        if let Some(def) = self.pick_default_enemy_target() {
                            self.target_cursor = def;
                        }
                        return;
                    }

                    if cmd == BattleCmd::Capture {
                        if let Some(eu) = self.unit(Side::Enemy, self.target_cursor) {
                            if eu.is_capture_ready() && *pods_left > 0 {
                                *pods_left -= 1;
                                let foe_short = eu.data.species.meta().short_name;
                                let foe_hp_pct = eu.hp_pct();
                                if let Some(au) = self.unit_mut(Side::Ally, ally_slot) {
                                    au.last_cmd = BattleCmd::Capture;
                                }
                                self.push_log(format!(
                                    "CONTAINMENT POD LOCKED ON {} ({}% HP)!",
                                    foe_short,
                                    foe_hp_pct
                                ));
                                sfx_out.push(Sfx::SuctionLaunch);
                                self.phase = BattlePhase::PodCaptureAnim {
                                    ally_slot,
                                    enemy_slot: self.target_cursor,
                                    ticks: 9,
                                };
                            } else {
                                let cutoff = eu.capture_hp_cutoff();
                                self.push_log(format!(
                                    "HP TOO HIGH ({}/{})! CHIP TO <={} HP (40%)!",
                                    eu.cur_hp, eu.max_hp, cutoff
                                ));
                                sfx_out.push(Sfx::Hurt);
                            }
                        }
                    } else {
                        self.commit_ally_command(ally_slot, cmd, Some(self.target_cursor));
                    }
                }
            }
            _ => {}
        }
    }

    fn move_enemy_target_cursor(&mut self, up: bool, down: bool, left: bool, right: bool) {
        let cur = self.target_cursor;
        let r = cur % 3;
        let c = cur / 3; // 0 = Front Col, 1 = Back Col
        let desired = if up {
            c * 3 + (r + 2) % 3
        } else if down {
            c * 3 + (r + 1) % 3
        } else if left || right {
            (1 - c) * 3 + r
        } else {
            cur
        };

        if self.unit(Side::Enemy, desired).is_some() {
            self.target_cursor = desired;
        } else {
            let living = self.living_slots(Side::Enemy);
            if !living.is_empty() {
                let pos = living.iter().position(|&s| s == cur).unwrap_or(0);
                let next = if up || left {
                    (pos + living.len() - 1) % living.len()
                } else {
                    (pos + 1) % living.len()
                };
                self.target_cursor = living[next];
            }
        }
    }

    /// Writes surviving ally HP & per-unit action memory back to persistent run squad
    pub fn sync_back_to_squad(&self, squad: &mut [Option<CreatureInstance>; 6]) {
        for i in 0..6 {
            if let Some(ref mut slot_inst) = squad[i] {
                if let Some(ref bu) = self.allies[i] {
                    slot_inst.cur_hp = bu.cur_hp.max(1); // Survivors & KO'd allies recover at 1 HP post-win!
                    slot_inst.last_cmd = bu.data.last_cmd;
                } else {
                    slot_inst.cur_hp = 1;
                }
            }
        }
    }

    // =========================================================================
    // 400x300 64-COLOR VGA BATTLE RENDERER (100% INTEGER PIXEL GRID)
    // =========================================================================
    pub fn render(&self, fb: &mut Framebuffer, assets: &AssetBank, pods_left: usize) {
        fb.blit_parallax_bg(&assets.bg_playfield, (self.zone_num as i32) * 80, 0, 196);

        // 1. Top Turn-Order Timeline Bar (y = 0..23)
        fb.rect_bevel(0, 0, 400, 23, 2, 4, 1, 1);
        fb.draw_text("ORDER:", 6, 8, 6, Some(1), 1);

        let timeline = self.preview_timeline();
        for (i, &(side, slot)) in timeline.iter().take(8).enumerate() {
            let tx = 44 + (i as i32) * 26;
            let ty = 3;
            let is_now = i == 0;
            let fill = match (side, is_now) {
                (Side::Ally, true) => 33,
                (Side::Ally, false) => 3,
                (Side::Enemy, true) => 37,
                (Side::Enemy, false) => 19,
            };
            let hi = if is_now { 15 } else { 4 };
            fb.rect_bevel(tx, ty, 24, 17, fill, hi, 1, 1);
            if let Some(u) = self.unit(side, slot) {
                let tag: String = u.data.species.meta().short_name.chars().take(3).collect();
                fb.draw_text(&tag, tx + 3, ty + 5, if is_now { 15 } else { 7 }, Some(1), 1);
            }
        }

        fb.draw_text(
            &format!("PODS:{}", pods_left),
            258,
            3,
            if pods_left > 0 { 15 } else { 38 },
            Some(1),
            1,
        );
        let auto_txt = if self.turbo_auto {
            "[T]:TURBO ON"
        } else {
            "[T]:AUTO OFF"
        };
        fb.draw_text(
            auto_txt,
            316,
            3,
            if self.turbo_auto { 25 } else { 34 },
            Some(1),
            1,
        );

        let status_right = if self.chain_depth > 1 {
            format!("CHAIN X{}!", self.chain_depth)
        } else if self.is_boss_battle {
            format!("Z{} BOSS!", self.zone_num)
        } else {
            format!("Z{} WILD PACK", self.zone_num)
        };
        fb.draw_text(&status_right, 258, 13, 42, Some(1), 1);

        // Column headers for 3x2 Tactical Grid (cleanly in y = 24..32 band!)
        fb.draw_text("ALLY BACK", 22, 24, 5, Some(1), 1);
        fb.draw_text("ALLY FRNT", 112, 24, 6, Some(1), 1);
        fb.draw_text("FOE FRNT", 236, 24, 38, Some(1), 1);
        fb.draw_text("FOE BACK", 326, 24, 5, Some(1), 1);

        let acting_ally = match self.phase {
            BattlePhase::WaitingAllyCommand { slot }
            | BattlePhase::WaitingEnemyTarget {
                ally_slot: slot, ..
            } => Some(slot),
            _ => None,
        };
        let targeted_enemy = match self.phase {
            BattlePhase::WaitingEnemyTarget { .. } => Some(self.target_cursor),
            _ => None,
        };

        // 2. Render 3x2 Ally Formation (Left) & 3x2 Enemy Formation (Right)
        for side in [Side::Ally, Side::Enemy] {
            for slot in 0..6 {
                let is_acting = side == Side::Ally && acting_ally == Some(slot);
                let is_target = side == Side::Enemy && targeted_enemy == Some(slot);
                self.render_formation_slot(fb, assets, side, slot, is_acting, is_target);
            }
        }

        // Integer Bresenham Chain-Reaction Connector Arcs!
        for arc in &self.arcs {
            let (x0, y0) = card_center(arc.from.0, arc.from.1);
            let (x1, y1) = card_center(arc.to.0, arc.to.1);
            let mid_x = (x0 + x1) / 2;
            let mid_y = ((y0 + y1) / 2 - 10).max(32);
            fb.line(x0, y0, mid_x, mid_y, 1);
            fb.line(mid_x, mid_y, x1, y1, 1);
            fb.line(x0, y0 - 1, mid_x, mid_y - 1, arc.col);
            fb.line(mid_x, mid_y - 1, x1, y1 - 1, arc.col);
            fb.circle_fill(x1, y1, 3, arc.col);
        }

        // Thrown Containment Pod Animation!
        if let BattlePhase::PodCaptureAnim {
            ally_slot,
            enemy_slot,
            ticks,
        } = self.phase
        {
            let (ax, ay) = card_center(Side::Ally, ally_slot);
            let (ex, ey) = card_center(Side::Enemy, enemy_slot);
            let prog = (9 - ticks).clamp(0, 9);
            let px = ax + ((ex - ax) * prog) / 9;
            let arc_lift = (prog * (9 - prog)) * 2;
            let py = ay + ((ey - ay) * prog) / 9 - arc_lift;
            fb.blit_sprite(&assets.spr_pod_12, px - 6, py - 6, false, false, None);
            if ticks <= 4 {
                let (rx, ry, rw, rh) = card_rect(Side::Enemy, enemy_slot);
                fb.rect_dither(rx + 2, ry + 2, rw - 4, rh - 4, 34, 2);
                fb.draw_text("CAPTURED!", rx + 15, ry + 20, 15, Some(1), 1);
            }
        }

        // Floating Damage / Heal / Buff Badges
        for pop in &self.pops {
            let (rx, ry, _, _) = card_rect(pop.side, pop.slot);
            let lift = (11 - pop.ticks).max(0);
            fb.rect_fill(rx + 26, (ry + 12 - lift).max(28), (pop.text.len() as i32) * 6 + 4, 10, 1);
            fb.draw_text(&pop.text, rx + 28, (ry + 13 - lift).max(29), pop.col, None, 1);
        }

        // 3. 2-Line Chain-Reaction Feed (y = 196..222)
        fb.rect_bevel(0, 196, 400, 26, 16, 4, 1, 1);
        let l1: String = self.log_line_1.chars().take(64).collect();
        let l2: String = self.log_line_2.chars().take(64).collect();
        fb.draw_text(&l1, 6, 200, 6, Some(1), 1);
        fb.draw_text(&l2, 6, 210, 15, Some(1), 1);

        // 4. Bottom Command Box (x = 0..136) + Live Trait/Target Inspector (x = 136..400)
        self.render_bottom_tactical_hud(fb, pods_left);
    }

    fn render_formation_slot(
        &self,
        fb: &mut Framebuffer,
        assets: &AssetBank,
        side: Side,
        slot: usize,
        is_acting: bool,
        is_target: bool,
    ) {
        let (x, y, w, h) = card_rect(side, slot);
        let Some(u) = self.unit(side, slot) else {
            // Subtle recessed formation pad for empty 3x2 slot
            fb.rect_dither(x + 2, y + 4, w - 4, h - 8, 1, 1);
            fb.rect_outline(x + 2, y + 4, w - 4, h - 8, 2);
            return;
        };

        let meta = u.data.species.meta();
        let cap_ready = side == Side::Enemy && u.is_capture_ready();

        let (fill, border_hi) = if is_target {
            (28, 15) // Targeted enemy highlight
        } else if is_acting {
            (18, 35) // Acting ally highlight
        } else if cap_ready {
            (22, 15) // Golden border when wild monster is chipped into <=40% HP capture window!
        } else {
            (2, 4)
        };
        fb.rect_bevel(x, y, w, h, fill, border_hi, 1, 1);

        // Card Header (<= 12 chars = 72px inside 76px inner width!)
        let prefix = if u.data.is_alpha { "*" } else { "" };
        let header = format!("{}{} L{}", prefix, meta.short_name, u.data.level);
        let h_col = if u.data.is_alpha {
            15
        } else {
            meta.affinity.color()
        };
        fb.draw_text(&header, x + 4, y + 3, h_col, Some(1), 1);

        // Grounded Stone Pedestal at y + 37 (zero floating feet: 24x24 sprite sits at y + 13..37!)
        fb.rect_fill(x + 3, y + 37, 28, 3, 4);
        fb.rect_fill(x + 3, y + 39, 28, 1, 1);
        let spr = &assets.monsters[u.data.species as usize];
        let flip_x = side == Side::Enemy;
        let flash = if u.hit_flash > 0 { Some(7) } else { None };
        fb.blit_sprite(spr, x + 5, y + 13, flip_x, false, flash);

        // Right-side Tactical Status & HP Readout (x + 33..80 = 47px = 7..8 chars!)
        let mut status_str = String::new();
        if u.defending_hold_fire {
            status_str.push_str("HOLD ");
        } else if u.provoking {
            status_str.push_str("TAUNT");
        } else if u.poison > 0 {
            status_str.push_str(&format!("PSN{}", u.poison));
        } else if u.atk_stages > 0 || u.int_stages > 0 || u.def_stages > 0 || u.spd_stages > 0 {
            let tot = u.atk_stages + u.int_stages + u.def_stages + u.spd_stages;
            status_str.push_str(&format!("+{}ST", tot));
        } else {
            status_str.push_str(meta.affinity.short_tag());
        }
        let st_trim: String = status_str.chars().take(7).collect();
        let st_col = if u.defending_hold_fire {
            34
        } else if u.poison > 0 {
            30
        } else {
            6
        };
        fb.draw_text(&st_trim, x + 34, y + 15, st_col, Some(1), 1);

        if cap_ready {
            fb.draw_text("<=40%CAP", x + 32, y + 27, 15, Some(1), 1);
        } else if u.shield > 0 {
            let s_txt = format!("SHD+{}", u.shield);
            let s_trim: String = s_txt.chars().take(7).collect();
            fb.draw_text(&s_trim, x + 34, y + 27, 34, Some(1), 1);
        } else {
            let hp_txt = format!("{}/{}", u.cur_hp, u.max_hp);
            let hp_trim: String = hp_txt.chars().take(7).collect();
            fb.draw_text(&hp_trim, x + 34, y + 27, 7, Some(1), 1);
        }

        // Full-Width HP & Shield Bar (x + 4..80, y + 42..47, 76x5 px)
        let bar_w = 76;
        fb.rect_fill(x + 4, y + 42, bar_w, 5, 1);
        fb.rect_fill(x + 5, y + 43, bar_w - 2, 3, 36);
        let fill_w = ((bar_w - 2) * u.cur_hp.max(0)) / u.max_hp.max(1);
        let hp_col = if cap_ready {
            15 // Gold bar signals 100% Capture Ready!
        } else if u.hp_pct() > 40 {
            25
        } else {
            14
        };
        if fill_w > 0 {
            fb.rect_fill(x + 5, y + 43, fill_w, 3, hp_col);
        }
        if u.shield > 0 {
            let shd_w = (((bar_w - 2) * u.shield) / u.max_hp.max(1)).clamp(2, bar_w - 2);
            fb.rect_fill(x + 5, y + 42, shd_w, 1, 35);
        }
    }

    fn render_bottom_tactical_hud(&self, fb: &mut Framebuffer, pods_left: usize) {
        // Left Command Menu (x = 0..136, y = 222..300)
        fb.rect_bevel(0, 222, 136, 78, 2, 4, 1, 1);
        let cap_label = format!("CAPTURE ({})", pods_left);
        let labels: [&str; 5] = [
            "ATTACK",
            "CAST GEM",
            "DEFEND/HOLD",
            "PROVOKE",
            &cap_label,
        ];
        let in_cmd = matches!(
            self.phase,
            BattlePhase::WaitingAllyCommand { .. } | BattlePhase::WaitingEnemyTarget { .. }
        );

        for (i, label) in labels.iter().enumerate() {
            let cy = 226 + (i as i32) * 12;
            let sel = in_cmd && self.cmd_cursor == i;
            if sel {
                fb.rect_fill(4, cy - 1, 128, 10, 18);
                fb.draw_text(">", 6, cy, 15, Some(1), 1);
            }
            let col = if i == 4 && (self.is_boss_battle || pods_left == 0) {
                5
            } else if sel {
                15
            } else {
                7
            };
            fb.draw_text(label, 16, cy, col, Some(1), 1);
        }
        fb.draw_text("[Z/ENT]OK [X]BACK", 8, 288, 34, Some(1), 1);

        // Right Inspector Box (x = 136..400, y = 222..300 -> 42 chars inner width!)
        fb.rect_bevel(136, 222, 264, 78, 2, 4, 1, 1);

        match self.phase {
            BattlePhase::WaitingEnemyTarget { ally_slot, cmd } => {
                if let Some(eu) = self.unit(Side::Enemy, self.target_cursor) {
                    let em = eu.data.species.meta();
                    let adv_str = if let Some(au) = self.unit(Side::Ally, ally_slot) {
                        let mult = au.data.species.meta().affinity.damage_mult_pct(em.affinity);
                        if mult > 100 {
                            " [+25% ADV!]"
                        } else if mult < 100 {
                            " [-15% RES]"
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };
                    let title = format!(
                        "TARGET: {} LV.{}{}",
                        em.name, eu.data.level, adv_str
                    );
                    let t_trim: String = title.chars().take(41).collect();
                    fb.draw_text(&t_trim, 142, 227, 15, Some(1), 1);

                    if cmd == BattleCmd::Capture {
                        if eu.is_capture_ready() {
                            fb.draw_text(
                                &format!(
                                    "HP: {}/{} ({}%) - 100% CAPTURE READY!",
                                    eu.cur_hp,
                                    eu.max_hp,
                                    eu.hp_pct()
                                ),
                                142,
                                239,
                                25,
                                Some(1),
                                1,
                            );
                        } else {
                            fb.draw_text(
                                &format!(
                                    "HP: {}/{} ({}%) - CHIP TO <={} HP!",
                                    eu.cur_hp,
                                    eu.max_hp,
                                    eu.hp_pct(),
                                    eu.capture_hp_cutoff()
                                ),
                                142,
                                239,
                                38,
                                Some(1),
                                1,
                            );
                        }
                    } else {
                        fb.draw_text(
                            &format!(
                                "HP:{}/{} ATK:{} DEF:{} INT:{} SPD:{}",
                                eu.cur_hp,
                                eu.max_hp,
                                eu.eff_atk(),
                                eu.eff_def(),
                                eu.eff_int(),
                                eu.eff_spd()
                            ),
                            142,
                            239,
                            7,
                            Some(1),
                            1,
                        );
                    }

                    fb.draw_text(
                        &format!("FOE TRAIT [{}]:", em.trait_name),
                        142,
                        253,
                        34,
                        Some(1),
                        1,
                    );
                    fb.draw_text(em.trait_desc_1, 142, 265, 6, Some(1), 1);
                    fb.draw_text(em.trait_desc_2, 142, 277, 6, Some(1), 1);
                }
            }

            BattlePhase::WaitingAllyCommand { slot } => {
                if let Some(au) = self.unit(Side::Ally, slot) {
                    let am = au.data.species.meta();
                    let gm = au.data.spell_gem.meta();
                    fb.draw_text(
                        &format!(
                            "{} LV.{} [{}]",
                            am.name,
                            au.data.level,
                            am.affinity.label()
                        ),
                        142,
                        227,
                        15,
                        Some(1),
                        1,
                    );
                    fb.draw_text(
                        &format!(
                            "ATK:{} DEF:{} INT:{} SPD:{}",
                            au.eff_atk(),
                            au.eff_def(),
                            au.eff_int(),
                            au.eff_spd()
                        ),
                        142,
                        238,
                        7,
                        Some(1),
                        1,
                    );

                    // Context-sensitive preview based on highlighted command!
                    match BattleCmd::ALL[self.cmd_cursor] {
                        BattleCmd::CastGem => {
                            fb.draw_text(
                                &format!("GEM: {}", gm.name),
                                142,
                                252,
                                34,
                                Some(1),
                                1,
                            );
                            fb.draw_text(gm.desc_1, 142, 264, 25, Some(1), 1);
                            fb.draw_text(gm.desc_2, 142, 276, 6, Some(1), 1);
                        }
                        BattleCmd::DefendHold => {
                            fb.draw_text("STANCE: DEFEND & HOLD FIRE", 142, 252, 34, Some(1), 1);
                            fb.draw_text("TAKES 50% DMG & PAUSES OFFENSIVE TRAITS", 142, 264, 25, Some(1), 1);
                            fb.draw_text("SO YOU WON'T OVERKILL <=40% WILD TARGETS!", 142, 276, 6, Some(1), 1);
                        }
                        BattleCmd::Capture => {
                            fb.draw_text("CONTAINMENT POD (ZERO RNG!)", 142, 252, 15, Some(1), 1);
                            fb.draw_text("CHIP ANY WILD SPECIMEN TO 1%-40% HP FOR", 142, 264, 25, Some(1), 1);
                            fb.draw_text("100% GUARANTEED CAPTURE! (GOLD HP BAR)", 142, 276, 6, Some(1), 1);
                        }
                        _ => {
                            fb.draw_text(
                                &format!("TRAIT: {}", am.trait_name),
                                142,
                                252,
                                34,
                                Some(1),
                                1,
                            );
                            fb.draw_text(am.trait_desc_1, 142, 264, 6, Some(1), 1);
                            fb.draw_text(am.trait_desc_2, 142, 276, 6, Some(1), 1);
                        }
                    }
                }
            }

            _ => {
                fb.draw_text("EXECUTING TRAIT-CHAIN TIMELINE...", 142, 234, 15, Some(1), 1);
                fb.draw_text(
                    "PRESS [T] ANYTIME TO TOGGLE TURBO AUTO!",
                    142,
                    250,
                    34,
                    Some(1),
                    1,
                );
                fb.draw_text(
                    "GOLD HP BAR = <=40% HP CAPTURE READY!",
                    142,
                    266,
                    25,
                    Some(1),
                    1,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siralim_trait_chain_cascade_and_hold_fire_capture_control() {
        // Build a 3-creature synergy loop:
        // Slot 0 (Front-Top): DewNewt (Soothing Dew heals adjacent on Defend/Act)
        // Slot 1 (Front-Mid): SporeMorel (Mycelial Bloom deals dmg + Poison when ally healed)
        // Slot 3 (Back-Top): VenomViper (Neurotoxin Fang gains +ATK when enemy poisoned)
        let mut allies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        allies[0] = Some(CreatureInstance::new(SpeciesId::DewNewt, 4, false));
        allies[1] = Some(CreatureInstance::new(SpeciesId::SporeMorel, 4, false));
        allies[3] = Some(CreatureInstance::new(SpeciesId::VenomViper, 4, false));

        let mut enemies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        enemies[0] = Some(CreatureInstance::new(SpeciesId::BriarToad, 4, false));

        let mut bs = BattleState::new(
            &allies,
            enemies,
            false,
            1,
            "TEST WILD PACK".to_string(),
            42,
        );
        let initial_foe_hp = bs.unit(Side::Enemy, 0).unwrap().cur_hp;

        // DewNewt Defends -> heals adjacent SporeMorel & VenomViper -> triggers SporeMorel Mycelial Bloom
        // -> damages & Poisons enemy BriarToad -> triggers VenomViper +20% ATK!
        bs.commit_ally_command(0, BattleCmd::DefendHold, None);
        let mut sfx = Vec::new();
        let mut pods = 3usize;
        for _ in 0..80 {
            bs.tick_animations_and_flow(&mut pods, &mut sfx);
            if matches!(bs.phase, BattlePhase::WaitingAllyCommand { .. }) {
                break;
            }
        }

        let foe_after = bs.unit(Side::Enemy, 0).unwrap().cur_hp;
        assert!(
            foe_after < initial_foe_hp,
            "Cascading Heal -> Mycelial Bloom chain must damage enemy"
        );
        assert!(
            bs.unit(Side::Ally, 3).unwrap().atk_stages > 0,
            "VenomViper must gain +ATK stage when Mycelial Bloom poisons enemy"
        );

        // Now verify DEFEND (HOLD FIRE) on SporeMorel suppresses Mycelial Bloom so players can hold fire for <=40% captures!
        let hp_before_hold = bs.unit(Side::Enemy, 0).unwrap().cur_hp;
        bs.unit_mut(Side::Ally, 1).unwrap().defending_hold_fire = true;
        bs.commit_ally_command(0, BattleCmd::DefendHold, None);
        while !bs.pending_steps.is_empty() {
            let step = bs.pending_steps.pop_front().unwrap();
            bs.execute_chain_step(step, &mut sfx);
        }
        assert_eq!(
            bs.unit(Side::Enemy, 0).unwrap().cur_hp,
            hp_before_hold,
            "SporeMorel in DEFEND/HOLD stance must not snipe capture targets!"
        );
    }

    #[test]
    fn test_deterministic_40_percent_capture_threshold() {
        let mut allies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        allies[0] = Some(CreatureInstance::new(SpeciesId::DewNewt, 4, false));

        let mut enemies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        enemies[0] = Some(CreatureInstance::new(SpeciesId::VoltaicEel, 4, true));

        let mut bs = BattleState::new(&allies, enemies, false, 1, "ALPHA EEL".to_string(), 7);
        bs.phase = BattlePhase::WaitingEnemyTarget {
            ally_slot: 0,
            cmd: BattleCmd::Capture,
        };
        bs.target_cursor = 0;

        let mut pods = 2usize;
        let mut sfx = Vec::new();

        // At 100% HP (>40%), Capture must refuse without wasting a Pod!
        bs.handle_player_input(false, false, false, false, true, false, false, &mut pods, &mut sfx);
        assert_eq!(pods, 2, "Pod must never be wasted when target is >40% HP");

        // Chip target down to 35% HP -> deterministic 100% capture!
        let max_hp = bs.unit(Side::Enemy, 0).unwrap().max_hp;
        bs.unit_mut(Side::Enemy, 0).unwrap().cur_hp = (max_hp * 35) / 100;
        assert!(bs.unit(Side::Enemy, 0).unwrap().is_capture_ready());

        bs.handle_player_input(false, false, false, false, true, false, false, &mut pods, &mut sfx);
        assert_eq!(pods, 1, "Pod consumed on <=40% HP capture");
        for _ in 0..15 {
            bs.tick_animations_and_flow(&mut pods, &mut sfx);
        }
        assert_eq!(bs.captured_units.len(), 1);
        assert_eq!(bs.captured_units[0].species, SpeciesId::VoltaicEel);
        assert!(bs.captured_units[0].is_alpha);
    }

    #[test]
    fn test_per_unit_last_action_memory_across_turns_and_battles() {
        let mut squad: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        squad[0] = Some(CreatureInstance::new(SpeciesId::DewNewt, 5, false));
        squad[1] = Some(CreatureInstance::new(SpeciesId::StarCarp, 5, false));

        // High-HP punching bag so multi-turn memory can be exercised cleanly
        let mut boss_bag = CreatureInstance::new(SpeciesId::BogGolem, 15, true);
        boss_bag.cur_hp = boss_bag.max_hp();
        let mut enemies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        enemies[0] = Some(boss_bag);

        let mut bs = BattleState::new(&squad, enemies, false, 1, "MEMORY TEST".to_string(), 99);
        let mut pods = 3usize;
        let mut sfx = Vec::new();

        let mut expected_cmd = [BattleCmd::Attack, BattleCmd::Attack];
        let desired_cmd = [BattleCmd::CastGem, BattleCmd::Provoke];
        let mut turns_seen = [0usize, 0usize];

        for _ in 0..400 {
            bs.tick_animations_and_flow(&mut pods, &mut sfx);
            if let BattlePhase::WaitingAllyCommand { slot } = bs.phase {
                assert!(slot < 2);
                assert_eq!(
                    bs.cmd_cursor,
                    expected_cmd[slot] as usize,
                    "Slot {} turn {} must default to its own last performed action",
                    slot,
                    turns_seen[slot]
                );
                bs.commit_ally_command(slot, desired_cmd[slot], None);
                expected_cmd[slot] = desired_cmd[slot];
                turns_seen[slot] += 1;
                if turns_seen[0] >= 2 && turns_seen[1] >= 2 {
                    break;
                }
            }
        }

        assert!(
            turns_seen[0] >= 2 && turns_seen[1] >= 2,
            "Both allies must take at least 2 turns to verify independent action memory"
        );

        // Sync back to squad and verify a brand-new battle preserves each unit's last combat action!
        bs.sync_back_to_squad(&mut squad);
        assert_eq!(squad[0].as_ref().unwrap().last_cmd, BattleCmd::CastGem as u8);
        assert_eq!(squad[1].as_ref().unwrap().last_cmd, BattleCmd::Provoke as u8);

        let mut enemies2: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        enemies2[0] = Some(CreatureInstance::new(SpeciesId::BogGolem, 15, true));
        let mut bs2 = BattleState::new(&squad, enemies2, false, 1, "MEMORY TEST 2".to_string(), 101);
        for _ in 0..80 {
            bs2.tick_animations_and_flow(&mut pods, &mut sfx);
            if let BattlePhase::WaitingAllyCommand { slot } = bs2.phase {
                assert_eq!(
                    bs2.cmd_cursor,
                    desired_cmd[slot] as usize,
                    "New battle must initialize unit cursor from its last performed action"
                );
                break;
            }
        }
    }
}
