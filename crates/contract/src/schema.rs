//! Published JSON Schemas for the frozen API contract.
//!
//! Keep these definitions alongside `lib.rs`; CI regenerates the checked-in
//! schema files and checks the canonical JSON examples against the wire types.

use serde_json::{json, Value};

pub fn tick_request_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://galactic-conquest.invalid/schema/tick-request.json",
        "title": "TickRequest",
        "type": "object",
        "additionalProperties": false,
        "required": ["api_version", "tick", "game_year", "deadline_ms", "match", "your_civilization", "visible_state"],
        "properties": {
            "api_version": {"const": "0.4"},
            "tick": {"type": "integer", "minimum": 0},
            "game_year": {"type": "integer", "minimum": 0},
            "deadline_ms": {"type": "integer", "minimum": 1},
            "match": {"$ref": "#/$defs/MatchView"},
            "your_civilization": {"$ref": "#/$defs/CivilizationView"},
            "visible_state": {"$ref": "#/$defs/VisibleState"},
            "last_tick_report": {"$ref": "#/$defs/TickReport"}
        },
        "$defs": {
            "MatchView": {"type":"object", "additionalProperties":false, "required":["state","systems_total","planets_total","civs_alive"], "properties":{
                "state":{"enum":["lobby","smoke","armed","running","paused","finished"]}, "systems_total":{"type":"integer","minimum":0}, "planets_total":{"type":"integer","minimum":0}, "civs_alive":{"type":"integer","minimum":0}}},
            "Resources": {"type":"object", "additionalProperties":false, "required":["credits","income_per_tick"], "properties":{"credits":{"type":"integer"},"income_per_tick":{"type":"number"}}},
            "Scores": {"type":"object", "additionalProperties":false, "required":["territory","military","economy","population"], "properties":{"territory":{"type":"number"},"military":{"type":"number"},"economy":{"type":"number"},"population":{"type":"number"}}},
            "Multipliers": {"type":"object", "additionalProperties":false, "required":["military","economy","population"], "properties":{"military":{"type":"number"},"economy":{"type":"number"},"population":{"type":"number"}}},
            "CivilizationView": {"type":"object", "additionalProperties":false, "required":["id","archetype","civilization_score","resources","score_breakdown","multipliers"], "properties":{"id":{"type":"string"},"archetype":{"type":"string"},"civilization_score":{"type":"number"},"resources":{"$ref":"#/$defs/Resources"},"score_breakdown":{"$ref":"#/$defs/Scores"},"multipliers":{"$ref":"#/$defs/Multipliers"}}},
            "VisibleState": {"type":"object", "additionalProperties":false, "required":["owned_planets","systems","your_fleets","visible_enemy_fleets","build_queue"], "properties":{"owned_planets":{"type":"array","items":{"$ref":"#/$defs/OwnedPlanet"}},"systems":{"type":"array","items":{"$ref":"#/$defs/SystemIntel"}},"your_fleets":{"type":"array","items":{"$ref":"#/$defs/YourFleet"}},"visible_enemy_fleets":{"type":"array","items":{"$ref":"#/$defs/EnemyFleet"}},"build_queue":{"type":"array","items":{"$ref":"#/$defs/BuildQueueItem"}}}},
            "Richness": {"enum":["poor","average","rich","exceptional"]},
            "OwnedPlanet": {"type":"object","additionalProperties":false,"required":["planet_id","system_id","richness","base_yield","pop","pop_capacity","garrison","garrison_target"],"properties":{"planet_id":{"type":"string"},"system_id":{"type":"string"},"richness":{"$ref":"#/$defs/Richness"},"base_yield":{"type":"integer","minimum":0},"pop":{"type":"integer","minimum":0},"pop_capacity":{"type":"integer","minimum":0},"garrison":{"type":"integer","minimum":0},"garrison_target":{"type":"integer","minimum":0}}},
            "SystemIntel": {"type":"object","additionalProperties":false,"required":["system_id","tier","lanes"],"properties":{"system_id":{"type":"string"},"tier":{"enum":["snapshot","adjacent_fog","scouted","owned"]},"last_seen_tick":{"type":"integer","minimum":0},"planets":{"type":"array","items":{"$ref":"#/$defs/PlanetSnapshot"}},"lanes":{"type":"array","items":{"$ref":"#/$defs/LaneIntel"}}}},
            "PlanetSnapshot": {"type":"object","additionalProperties":false,"required":["planet_id","richness"],"properties":{"planet_id":{"type":"string"},"richness":{"$ref":"#/$defs/Richness"},"owner":{"type":["string","null"]},"garrison":{"type":["integer","null"],"minimum":0}}},
            "LaneIntel": {"type":"object","additionalProperties":false,"required":["lane_id","to","hazard"],"properties":{"lane_id":{"type":"string"},"to":{"type":"string"},"travel_ticks":{"type":"integer","minimum":1,"maximum":255},"travel_ticks_est":{"type":"array","items":{"type":"integer"},"minItems":2,"maxItems":2},"hazard":{"enum":["none","slow","void","nebula"]}}},
            "YourFleet": {"type":"object","additionalProperties":false,"required":["fleet_id","kind","busy"],"properties":{"fleet_id":{"type":"string"},"kind":{"enum":["war","scout"]},"strength":{"type":"number","minimum":0},"system_id":{"type":"string"},"in_transit":{"$ref":"#/$defs/InTransit"},"busy":{"type":"boolean"}}},
            "InTransit": {"type":"object","additionalProperties":false,"required":["lane_id","to","arrives_tick"],"properties":{"lane_id":{"type":"string"},"to":{"type":"string"},"arrives_tick":{"type":"integer","minimum":0}}},
            "EnemyFleet": {"type":"object","additionalProperties":false,"required":["fleet_id","owner","strength","system_id"],"properties":{"fleet_id":{"type":"string"},"owner":{"type":"string"},"strength":{"type":"integer","minimum":0},"system_id":{"type":"string"}}},
            "BuildQueueItem": {"type":"object","additionalProperties":false,"required":["planet_id","unit","quantity","spawns_tick"],"properties":{"planet_id":{"type":"string"},"unit":{"enum":["warship","scout"]},"quantity":{"type":"integer","minimum":0},"spawns_tick":{"type":"integer","minimum":0}}},
            "TickReport": {"type":"object","additionalProperties":false,"required":["tick","actions","events"],"properties":{"tick":{"type":"integer","minimum":0},"actions":{"type":"array","items":{"$ref":"#/$defs/ActionReport"}},"events":{"type":"array","items":{"$ref":"#/$defs/AgentEvent"}}}},
            "ActionReport": {"type":"object","additionalProperties":false,"required":["index","status"],"properties":{"index":{"type":"integer","minimum":0},"status":{"enum":["ok","rejected"]},"code":{"$ref":"#/$defs/ActionErrorCode"},"message":{"type":"string"}}},
            "ActionErrorCode": {"enum":["too_many_actions","unknown_action_type","invalid_field","target_not_found_or_not_visible","not_owner","fleet_busy","fleet_not_in_system","fleet_wrong_kind","conflicting_orders","no_path","insufficient_credits"]},
            "AgentEvent": {"oneOf":[{"type":"object","additionalProperties":false,"required":["type","planet_id","your_strength_before","your_strength_after"],"properties":{"type":{"const":"battle"},"planet_id":{"type":"string"},"winner":{"type":["string","null"]},"your_strength_before":{"type":"number"},"your_strength_after":{"type":"number"}}},{"type":"object","additionalProperties":false,"required":["type","planet_id"],"properties":{"type":{"const":"planet_colonized"},"planet_id":{"type":"string"}}},{"type":"object","additionalProperties":false,"required":["type","planet_id"],"properties":{"type":{"const":"planet_captured"},"planet_id":{"type":"string"}}},{"type":"object","additionalProperties":false,"required":["type","civ"],"properties":{"type":{"const":"civ_eliminated"},"civ":{"type":"string"}}}]}
        }
    })
}

pub fn agent_response_schema() -> Value {
    json!({
        "$schema":"https://json-schema.org/draft/2020-12/schema",
        "$id":"https://galactic-conquest.invalid/schema/agent-response.json",
        "title":"AgentResponse", "type":"object", "required":["tick","actions"],
        "properties":{"tick":{"type":"integer","minimum":0},"actions":{"type":"array","maxItems":200,"items":{"$ref":"#/$defs/AgentAction"}}},
        "$defs":{"AgentAction":{"oneOf":[
            {"type":"object","additionalProperties":false,"required":["type","target","amount"],"properties":{"type":{"const":"invest_resources"},"target":{"enum":["military","economy","population"]},"amount":{"type":"integer","minimum":1}}},
            {"type":"object","additionalProperties":false,"required":["type","planet_id","unit","quantity"],"properties":{"type":{"const":"build"},"planet_id":{"type":"string"},"unit":{"enum":["warship","scout"]},"quantity":{"type":"integer","minimum":1}}},
            {"type":"object","additionalProperties":false,"required":["type","fleet_id","target_system"],"properties":{"type":{"const":"move_fleet"},"fleet_id":{"type":"string"},"target_system":{"type":"string"}}},
            {"type":"object","additionalProperties":false,"required":["type","fleet_id","target_planet"],"properties":{"type":{"const":"attack"},"fleet_id":{"type":"string"},"target_planet":{"type":"string"}}},
            {"type":"object","additionalProperties":false,"required":["type","fleet_id","target_system"],"properties":{"type":{"const":"scout"},"fleet_id":{"type":"string"},"target_system":{"type":"string"}}},
            {"type":"object","additionalProperties":false,"required":["type","fleet_id","target_planet"],"properties":{"type":{"const":"colonize"},"fleet_id":{"type":"string"},"target_planet":{"type":"string"}}}
        ]}}
    })
}

pub fn spectator_map_schema() -> Value {
    json!({
        "$schema":"https://json-schema.org/draft/2020-12/schema", "title":"SpectatorMap",
        "type":"object", "additionalProperties":false,
        "required":["api_version","systems","lanes","hazards","core","planets","civs"],
        "properties":{"api_version":{"const":"0.4"},"systems":{"type":"array","items":{"$ref":"#/$defs/System"}},"lanes":{"type":"array","items":{"$ref":"#/$defs/Lane"}},"hazards":{"type":"array","items":{"$ref":"#/$defs/HazardZone"}},"core":{"$ref":"#/$defs/Core"},"planets":{"type":"array","items":{"$ref":"#/$defs/Planet"}},"civs":{"type":"array","items":{"$ref":"#/$defs/Civ"}}},
        "$defs": {
            "System":{"type":"object","additionalProperties":false,"required":["id","x","y","region"],"properties":{"id":{"type":"string"},"x":{"type":"integer"},"y":{"type":"integer"},"region":{"type":"integer","minimum":0}}},
            "Lane":{"type":"object","additionalProperties":false,"required":["id","a","b","travel_ticks","hazard","blocked"],"properties":{"id":{"type":"string"},"a":{"type":"string"},"b":{"type":"string"},"travel_ticks":{"type":"integer","minimum":1,"maximum":8},"hazard":{"enum":["none","slow","void","nebula"]},"blocked":{"type":"boolean"}}},
            "HazardZone":{"type":"object","additionalProperties":false,"required":["id","kind","x","y","radius"],"properties":{"id":{"type":"string"},"kind":{"enum":["void","nebula"]},"x":{"type":"integer"},"y":{"type":"integer"},"radius":{"type":"integer","minimum":0}}},
            "Core":{"type":"object","additionalProperties":false,"required":["x","y","radius"],"properties":{"x":{"type":"integer"},"y":{"type":"integer"},"radius":{"type":"integer","minimum":0}}},
            "Planet":{"type":"object","additionalProperties":false,"required":["id","system_id","richness","base_yield"],"properties":{"id":{"type":"string"},"system_id":{"type":"string"},"richness":{"enum":["poor","average","rich","exceptional"]},"base_yield":{"type":"integer","minimum":0}}},
            "Civ":{"type":"object","additionalProperties":false,"required":["id","name","color"],"properties":{"id":{"type":"string"},"name":{"type":"string"},"color":{"type":"integer","minimum":0,"maximum":255}}}
        }
    })
}

pub fn spectator_tick_schema() -> Value {
    json!({
        "$schema":"https://json-schema.org/draft/2020-12/schema", "title":"SpectatorTick",
        "type":"object", "additionalProperties":false,
        "required":["api_version","tick","game_year","match","civs","planets","fleets","events","dominance"],
        "properties":{"api_version":{"const":"0.4"},"tick":{"type":"integer","minimum":0},"game_year":{"type":"integer","minimum":0},"match":{"$ref":"#/$defs/Match"},"civs":{"type":"array","items":{"$ref":"#/$defs/Civ"}},"planets":{"type":"array","items":{"$ref":"#/$defs/Planet"}},"fleets":{"type":"array","items":{"$ref":"#/$defs/Fleet"}},"events":{"type":"array","items":{"$ref":"#/$defs/Event"}},"dominance":{"$ref":"#/$defs/Dominance"}},
        "$defs": {
            "Match":{"type":"object","additionalProperties":false,"required":["state","time_remaining_ms","civs_alive","delay_ticks"],"properties":{"state":{"enum":["lobby","smoke","armed","running","paused","finished"]},"time_remaining_ms":{"type":"integer","minimum":0},"civs_alive":{"type":"integer","minimum":0},"delay_ticks":{"type":"integer","minimum":0}}},
            "Breakdown":{"type":"object","additionalProperties":false,"required":["territory","military","economy","population"],"properties":{"territory":{"type":"number"},"military":{"type":"number"},"economy":{"type":"number"},"population":{"type":"number"}}},
            "Civ":{"type":"object","additionalProperties":false,"required":["id","name","color","alive","civilization_score","score_breakdown","planets_owned","territory_share","war_strength","credits","archetype"],"properties":{"id":{"type":"string"},"name":{"type":"string"},"color":{"type":"integer","minimum":0,"maximum":255},"alive":{"type":"boolean"},"civilization_score":{"type":"number"},"score_breakdown":{"$ref":"#/$defs/Breakdown"},"planets_owned":{"type":"integer","minimum":0},"territory_share":{"type":"number","minimum":0,"maximum":1},"war_strength":{"type":"number","minimum":0},"credits":{"type":"integer","minimum":0},"archetype":{"type":["string","null"]},"elimination_tick":{"type":"integer","minimum":0}}},
            "Planet":{"type":"object","additionalProperties":false,"required":["planet_id","owner","garrison"],"properties":{"planet_id":{"type":"string"},"owner":{"type":["string","null"]},"garrison":{"type":"number","minimum":0}}},
            "Transit":{"type":"object","additionalProperties":false,"required":["lane_id","to","elapsed","total"],"properties":{"lane_id":{"type":"string"},"to":{"type":"string"},"elapsed":{"type":"integer","minimum":0},"total":{"type":"integer","minimum":1}}},
            "Fleet":{"type":"object","additionalProperties":false,"required":["fleet_id","owner","kind","strength"],"properties":{"fleet_id":{"type":"string"},"owner":{"type":"string"},"kind":{"enum":["war","scout"]},"strength":{"type":"number","minimum":0},"system_id":{"type":"string"},"in_transit":{"$ref":"#/$defs/Transit"}}},
            "Event":{"oneOf":[{"type":"object","additionalProperties":false,"required":["type","planet_id","parties","strength_before","strength_after"],"properties":{"type":{"const":"battle"},"planet_id":{"type":"string"},"parties":{"type":"array","items":{"type":"string"}},"winner":{"type":["string","null"]},"strength_before":{"type":"object","additionalProperties":{"type":"number"}},"strength_after":{"type":"object","additionalProperties":{"type":"number"}}}},{"type":"object","additionalProperties":false,"required":["type","planet_id","owner"],"properties":{"type":{"const":"planet_colonized"},"planet_id":{"type":"string"},"owner":{"type":"string"}}},{"type":"object","additionalProperties":false,"required":["type","planet_id","owner"],"properties":{"type":{"const":"planet_captured"},"planet_id":{"type":"string"},"owner":{"type":"string"}}},{"type":"object","additionalProperties":false,"required":["type","civ"],"properties":{"type":{"const":"civ_eliminated"},"civ":{"type":"string"}}}]},
            "Dominance":{"type":"object","additionalProperties":false,"required":["leader","share","threshold"],"properties":{"leader":{"type":["string","null"]},"share":{"type":"number","minimum":0,"maximum":1},"threshold":{"type":"number","minimum":0,"maximum":1}}}
        }
    })
}
