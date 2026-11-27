# The economy: legacy and now

What the pre-ECS economy (`script/Legacy`) did, what the ECS economy (`script/Modules/Economy` and
its neighbours) does instead, what is still to take over, and the principles the economy follows.

## Feature by feature

| Legacy | Now | |
|---|---|---|
| Items with mass and an `energy` value, the base of every price | Items with mass and a start price (`ItemDefs`) | Prices are set by hand; legacy derived them from energy |
| ~30 recipes in tiers (power, refinery, factory, drydock), some unreachable | 5 recipes (`RecipeDefs`): solar, refinery, foundry, colony, outpost upkeep; each stage pays about a fifth at start prices | Fewer, but balanced and all in play |
| Factory: one cycle at a time, output that doesn't fit is lost; uptime tracked | `ProductionSystem`: lines idle when outputs pile up unsold, or wait for room | Uptime/blocked not tracked yet |
| Trader: one order per unit, asks escrow goods, **bids escrow nothing**, settlement per unit, prices drift down by chance, bid raising dead | `MarketplaceSystem`: orders of any size, asks escrow goods **and bids escrow credits**, settlement out of escrow, emergent price with supply/demand and a pull to equilibrium | Credits can't be spent twice |
| Station orders posted after each cycle, priced from input energy × markup | `MarketMakerSystem`: bids to a stock target that covers resupply time, asks for all output, priced by stock pressure around a moving reference | |
| Ships reserve bid/ask offers | Claims per ask, bid, rock, patrol, held by the job | Same idea, released when a job ends (legacy leaked them) |
| Yields: cooldown 300 s, then 1 unit per 5 s | `YieldComponent` regrowing at a rate; ore spots for lasers | |
| Flows per entity summed per system, pressure scoring commented out | Recipe rates (`ProductionComponent:getRates`) drive the planner's station mix and miners' demand forecasts | Flows used for planning, not tracked live |
| Jobs rebuilt every 2 s (mine, transport, maraud, patrol); Think samples up to 4,000 jobs, straight-line times, a bug times the pickup leg to the destination | `JobBoardSystem` offers; captains think on a worker (`JobPlanner`) by profit per second with navigation estimates, claims of earlier captains counted | |
| Ship subtypes (Trader, Miner, Security) set by the job | No roles: equipment decides which jobs a ship can do; every ship is crewed, its captain decides | |
| Black market, BlackMarketTrader (a broken copy), Piracy recipe, CriminalThink | Marauders raid laden traders and miners, fence part of the value, buy fighters from loot | **No black market or stolen-goods trade yet** |
| Pirates at minimum disposition | Factions with relations, hostility from hits and raids, threat at stations, paid patrols, independents | |
| High/low attention per system (low did nothing) | One system simulated | **No attention levels yet** |
| Credits: 1e6–1e10 created at spawn, no sinks | Credits enter as colony income (and station stock); leave through ships bought (`shipCost`), wages to crews, patrol pay | Sources and sinks exist; their balance is to watch (`Testbeds/EconomySim`) |
| Planets as huge demand sinks bidding above any ask | Colonies consume steel and energy and earn income | |
| Debug panels and log tracing | Overlay in game; `Testbeds/EconomySim` for simulated hours with map, charts and a CSV | |

## Principles

1. **Value flows from consumption.** New credits enter where goods are used up (a colony's income),
   and leave where they buy something that isn't traded (a ship, wages, protection). Every other
   transaction moves credits between parties, out of escrow. Credits are never made to keep
   trading profitable (legacy planets did that).
2. **Every stage of a chain must pay at its own prices.** A recipe's outputs are worth more than its
   inputs at the items' reference prices, so no station bleeds for doing its job. When adding
   recipes, check the margin at start prices (legacy used `energy` × markup for this; we keep the
   check, not the formula).
3. **Prices emerge from orders, around a reference that moves with trades.** Stations quote from
   their stock against a target; scarcity raises bids, gluts lower asks. No random drift.
4. **Supply follows demand by rates, not guesses.** Station mixes, stock targets and miners' runs
   come from recipe rates and resupply times (the planner, `resupplyS`, demand forecasts).
5. **Agents decide, the world executes.** A captain chooses a job from a snapshot, off the main thread;
   flying, docking, trading and fighting run in systems on the ECS. Claims keep two captains off one
   lot and are released with the job.
6. **Equipment, not roles.** What a ship carries decides what it can do; the captain decides what it
   does. Careers, traits and memories will weigh in.
7. **Data over code.** Items, recipes, station kinds, ship kinds, crew roles, life events,
   formations: definitions, extended by adding entries.
8. **Watch it run.** Every economic change is checked in `Testbeds/EconomySim` over simulated hours
   (credits conserved but for known sources and sinks, prices settling, no stock running away).

## Still to take over

- **A black market**: stolen goods sold at pirate stations, a fence price, criminal captains who buy
  and sell there; marauders already raid and fence.
- **Attention levels**: systems far from the player simulated coarsely (no flight, jobs resolved by
  time), for many systems at once.
- **Factory uptime**: how long each line ran, waited for inputs, or idled on unsold output, in the
  sampler and the overlay.
- **Item value from content**: deriving start prices from what an item takes to make, so new recipes
  are balanced by construction.
- **More of the chain**: the legacy tiers (ship parts, hulls, stations) as goods, so shipyards build
  the ships factions buy instead of credits vanishing into `shipCost`.
