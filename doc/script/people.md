# People

The `People` module (`script/Modules/People`): the NPCs who man ships (and, later, stations), who
decide for them, and what they live through. Every AI ship is manned; none decides for itself.

## A person

A person is an entity (`NpcEntity`) of its own, apart from any ship:

- `PersonComponent`: their name, their traits (values by trait id, for the personality that will
  decide how they choose; none are read yet) and their morale (0 to 1).
- `CareerComponent`: who employs them (an entity with an account, such as a faction's leader; none
  for a self-employed owner), the posts they held (`history`), their experience by activity
  ("trade", "mining", "patrol", "raid") and what they were paid.
- `MemoryComponent`: the events they lived through that count, and an opinion (-1 to 1) of each
  person, faction or other entity an event was about.
- A wallet (`PlayerBankAccount`), and a `FactionMembershipComponent` when they belong to a faction.

## Posts, roles and command

A **post** is anything people serve on: a ship now, a station later. Its `CrewComponent` lists its
members and their roles and names its commander; each member's `AssignmentComponent` names the post
and the role. `PeopleManager` keeps the two in step (`assign`, `unassign`, `promote`).

Roles are data (`Shared/Definitions/CrewRoleDefs.lua`): a name, whether the role commands, a rank
(who takes over when command falls vacant) and a share of the post's earnings. A ship kind lists the
roles aboard (`ShipKindDefs.crew`): a hauler a captain, a mate and a hand; a fighter a captain alone.

The **commander decides**. The component types registered with
`PeopleManager:registerCommandComponent` live on the commander, not on the post: the economy's AI
(`EconomyAgentComponent`) is one, so `EconomyAgentSystem` runs the people who command ships, each
choosing jobs for the ship they command. When command passes (`setCommander`: a promotion, the
commander leaving or dying, later a mutiny), those components move to the new commander, and the
crew learn of it. Code that wants a ship's AI asks `PeopleManager:commandOf(ship, componentType)`.
Navigation orders, dockings and a ship's fight controller (`CombatAgentComponent`) stay on the ship:
they are how it is flown, not who decides.

## Employment and independents

- A **faction's ship** is owned by the faction's leader, whose account it trades from. Its crew are
  the leader's employees: each is paid their role's share of what the ship earns
  (`PeopleManager:reportEarnings`, which also adds experience). When a faction buys a ship
  (`SpawnerSystem`), it first hires people it has on its books but on no post, such as the crew of a
  lost ship, and makes new ones only for the roles left.
- An **independent** ship is owned by its captain (`PeopleManager:manIndependent`): it trades from
  the captain's wallet, and the captain employs and pays the rest of the crew. An independent has no
  faction: it is neutral to the factions, hostile to marauders, and its weapons' team is
  `FactionManager.INDEPENDENT_TEAM`. Star systems plan a few (`EconomyDefaults.independents`).

## What happens to people

Events are data (`Shared/Definitions/LifeEventDefs.lua`): each moves a person's morale and their
opinion of the event's subject, and the ones that count are remembered.
`PeopleManager:experience` reports one; `CrewSystem` reports what the world does to them:

- a ship destroyed: its crew survive, lose it (`ShipLost`, toward their employer) and are free for a
  new post with the same employer;
- a hit on a ship: everyone aboard is `Attacked`, toward the attacker's faction (or the attacker);
- a vacant command: the highest rank aboard takes it.

Pay (`Paid`, toward the employer), finished jobs (`JobDone`, toward the commander), hiring and
promotion are reported where they happen. Nothing reads morale, opinions or memories yet: they are
what traits, careers, crews' loyalty and mutiny will decide by.
