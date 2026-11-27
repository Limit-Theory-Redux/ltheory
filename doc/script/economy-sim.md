# Economy simulation

`Testbeds/EconomySim` runs a star system's economy for simulated hours, as fast as the machine
allows, to see how it behaves: whether prices settle, stock piles up or runs dry, credits leak or
grow, and what the agents spend their time on. It is the economy of the game (the default scenario's
star system and its planned economy, as `Testbeds/EconomyGame` builds it): the same stations, ships,
navigation, docking, mining and fights, only without the 3D rendering.

```
LTHEORY_BACKGROUND=1 LTHEORY_ECONOMY_HOURS=4 LTHEORY_ECONOMY_LOG=economy.csv ./bin/ltr.exe Testbeds/EconomySim
```

| Variable | Default | |
|---|---|---|
| `LTHEORY_ECONOMY_HOURS` | until closed | Simulated hours to run; the run then quits with a summary |
| `LTHEORY_ECONOMY_LOG` | none | A CSV file of every sample, and the summary |
| `LTHEORY_ECONOMY_SAMPLE` | 60 | Simulated seconds between samples |
| `LTHEORY_ECONOMY_SEED`, `LTHEORY_ECONOMY_SHIPS` | | As for `Testbeds/EconomyGame`: the universe seed, more ships |

## Fast, fixed steps

The simulation steps at a fixed 0.05 s, the step navigation is tested at, and several times a frame.
An application that sets `fixedSimStepS` has its simulation stages (PreSim, Sim, PostSim) see that
step instead of the frame's time (`Application:eventLoop` hands their handlers a `FixedStepEvent`),
and runs them `simStepsPerFrame` times a frame: the frame's stage subscriptions are noted and run
again after it. Events dispatched with a payload (a navigation order's end) are delivered once, as the
bus gives them. Input and rendering still run once a frame. The testbed adapts the steps a frame so a
frame takes about 0.15 s of real time, and shows how many times faster than real time it runs.

## What it shows

A top-down map of the system (x and z): planets as rings, stations as squares in their faction's
colour, ships as dots coloured by what their captain does (trading green, mining orange, fighting red,
idle grey). Four charts over the last samples: the average price of each item over the stations, the
stock of each item at the stations (their asks included), the agents by state, and the credits (the
stations', and every account's).

## What it logs

`EconomySampler` takes a sample every `LTHEORY_ECONOMY_SAMPLE` simulated seconds: each station's
credits (its bids included), stock and market price of each item; the agents by state, their trips and
what they earned and spent; each faction's treasury; and the credits of every account. With a CSV file
each sample is rows of `time,kind,name,key,value` (`kind` is `station`, `stock`, `price`, `agents`,
`faction` or `total`), easy to load into a spreadsheet or a script. At the end the summary (prices,
stock and credits from the first sample to the last) goes to the log and, as `#` lines, to the CSV.
