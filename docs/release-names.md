# Release names

Every Nexium release is a place on a mountain. A major version is a mountain,
taken in the order the fourteen 8000-metre peaks were first climbed; the
versions under it are the climb of that mountain: its camps, routes and faces
for minor versions, the members of its first-ascent expedition for patch
versions, and `Summit` for `X.0.0`. The 0.x line is the approach and the
camps of the first mountain, so 1.0.0 is standing on top of the mountain the
project has been on since 0.1.0. Decision 89 in `DECISIONS.md` records the
rule; this file is the reference, the ledger, and the plan.

## How to read a name

```
nx 0.7.0 (Annapurna: Camp V)
nx 1.0.0 (Annapurna: Summit)
nx 1.1.0 (Annapurna: Dutch Rib)
nx 1.2.0 (Annapurna: South Face)
nx 2.0.0 (Everest: Summit)
nx 2.1.0 (Everest: Khumbu Icefall)
```

The mountain says which major line you are on; the place says how far up it
is and what kind of release it was. The version number still carries the
ordering; the name carries the character.

## The rule

- **Major = mountain.** 0.x and 1.x are Annapurna (first 8000-metre peak
  climbed, 1950); 2.x Everest (1953), 3.x Nanga Parbat (1953), 4.x K2
  (1954), 5.x Cho Oyu (1954), 6.x Makalu (1955), 7.x Kangchenjunga (1955),
  8.x Manaslu (1956), 9.x Lhotse (1956), 10.x Gasherbrum II (1956), 11.x
  Broad Peak (1957), 12.x Gasherbrum I (1958), 13.x Dhaulagiri (1960), 14.x
  Shishapangma (1964). After the 8000ers come the Seven Summits, then the
  great north faces of the Alps; a range after those is a new decision.
- **`X.0.0` is `Mountain: Summit`.**
- **Minor = a place on the mountain**: a camp, a route, a face, a col, a
  feature. Each place below says what kind of release it fits. Short names:
  `Camp V`, not `the Sickle Glacier Couloir`.
- **Patch = a person from the mountain's first-ascent expedition.** Roster
  order by default, so nothing is forced onto a bug-fix release; when a
  patch plainly fits one person's story (a rescue, a removal, a
  documentation patch), take that one. When the roster runs out, the later
  climbers of that mountain follow.
- **0.x** are the approach and the camps of the first climb; if there are
  more minor releases than camps, the features of the summit route follow.
- The name fits the release; the release is never shaped to fit a name.

## How to pick

1. Write the changelog section first. The release's character is in it.
2. Find the place on the current mountain whose "use it for" matches;
   two candidates is fine, pick the shorter name.
3. For a patch, take the next unused person on the roster unless one
   clearly fits.
4. Put the name and one line of why under the version header, update the
   ledger here, and mark the pool entry used.

## Where the name appears

- `CHANGELOG.md`: the first line under the version header, in italics, with
  the reason after an em dash:

  ```
  ## [0.7.0] - 2026-09-19

  *Annapurna: Camp V* — the last camp, at 7,400 m; the summit push starts here.
  ```

  (`patchnotes` validates the header and ignores the line; the release
  script reads it.)
- The GitHub release title and the tag message: `Nexium v0.7.0 — Annapurna:
  Camp V` (`scripts/release_notes.py` writes the title).
- `nx version` and `nx doctor`: `nx 0.7.0 (Annapurna: Camp V)`; the name
  is `RELEASE_NAME` in `self/nx.nx`, set by the release commit with the
  version.

## Ledger

| version | name | why |
| --- | --- | --- |
| 0.1.0 | Annapurna: Miristi Khola | the gorge the 1950 expedition spent weeks finding a way through; the approach |
| 0.2.0 | Annapurna: Base Camp | where the expedition is staged: the installer, the spec, the roadmap |
| 0.2.1 | Annapurna: Herzog | patch (roster 1) |
| 0.3.0 | Annapurna: Camp I | the first camp on the mountain: the tools' standard library |
| 0.4.0 | Annapurna: Camp II | the language talks to the world: sockets, HTTP, threads |
| 0.5.0 | Annapurna: Camp III | other people can build on it: packages, editors, installers |
| 0.6.0 | Annapurna: Camp IV | the syntax settled for 1.0; the parser in Nexium |
| 0.6.1 | Annapurna: Lachenal | patch (roster 2) |
| 0.7.0 | Annapurna: Camp V | the last camp, at 7,400 m; the summit push starts here: the compiler builds itself |
| 0.8.0 | Annapurna: the Sickle | the exposed glacier crossing below the summit: the compiler in Nexium is the compiler, and builds from its own C with no Rust |
| 0.9.0 | Annapurna: Summit Ridge | the last ridge, nothing left but walking up: every tool in Nexium, the shipped `nx` the Nexium one, a harness and a fuzzer in Nexium, the stability policy and the tiers |
| 1.0.0 | Annapurna: Summit | the top of the mountain the project has been on since 0.1: the language stops changing under people's feet, the compiler is written in itself, and nothing but Nexium, one C file and a runtime header is left |
| 1.0.1 | Annapurna: Rébuffat | patch (roster 3): the loose ends after the summit; the fixes the tutorial found, the docs site and the Topo, eleven editors |
| 1.0.2 | Annapurna: Terray | patch (roster 4): the hotfix; the release binaries were built for the build machine's CPU and crashed on others |
| 1.0.3 | Annapurna: Schatz | patch (roster 6): the long-hidden bugs an outside review found, then the roads in: every way to install and the tools the roadmap's quick wins named |
| 1.1.0 | Annapurna: Dutch Rib | the safer line that became the everyday route: the ergonomics the compiler wanted (`?.`, tuple destructuring, `derive(Clone)`, iterators, slice patterns, guard facts, named format arguments, the intrinsics) |
| 1.2.0 | Annapurna: South Face | the great wall climbed by siege: memory safety without a garbage collector, the view rules |
| 1.2.1 | Annapurna: de Noyelle | patch (roster 9): the liaison officer, permits and diplomacy; the roads in: every registry and package format, completions and a manual page, provenance on every asset, the Topo as a course |
| 1.3.0 | Annapurna: North Face | the face of the first ascent, the original line followed through: the toolchain grown up; `nx fix`, `nx bench`, `nx debug` and `#line`, `--sanitize`, `if comptime`, the language server from the checker, incremental builds |

## The plan

Names pencilled in for the releases the roadmap describes. A plan is a
plan: if a release turns out to be something else, it takes the name that
fits and the pencilled one goes back in the pool.

| version | name | why |
| --- | --- | --- |
| 1.4.0 | Annapurna: the Sanctuary | the basin that holds everything and supplies every route: a standard library people stop supplementing |
| 1.5.0 | Annapurna: East Ridge | the long traverse over several summits: one source, many platforms |
| 1.6.0 | Annapurna: North-West Face | fast and light, no fixed ropes: the runtime release builds deserve |
| 1.7.0 | Annapurna: Annapurna II | the massif's second summit, joined to the main one by the long ridge: the seam, both ways (Python, Rust and more languages, in and out) |
| 1.8.0 | Annapurna: Gangapurna | a summit of the same massif, climbed beside the main line: a side theme (the GUI, the registry) |
| 1.x | Annapurna: Machapuchare | reserved for a release that is only deprecations: it stops short on purpose |
| 2.0.0 | Everest: Summit | the next mountain; a major, so the first breaking changes since 1.0 |
| 2.1.0 | Everest: Khumbu Icefall | the dangerous, unglamorous crossing right after the major: migrations, `nx fix`, the fallout |
| 2.2.0 | Everest: Western Cwm | the quiet valley: consolidation, nothing breaks |
| 2.3.0 | Everest: Lhotse Face | the long steep grind: performance and hardening |
| 2.4.0 | Everest: South Col | the last staging camp: everything in place for the next big feature |
| 2.5.0 | Everest: Hillary Step | the last technical crux of the line |

## The pools

Names to draw from, mountain by mountain, in the order the mountains are
used. Each place says what it was and what kind of release to use it for;
each person says who they were and what kind of patch fits. The histories
are the standard accounts, and a wrong detail here is fixed in this file.

Recurring kinds, so the glosses stay short: **groundwork** (CI, build,
infrastructure), **consolidation** (nothing new, everything tidier),
**headline** (the release people will remember), **hard crux** (small,
difficult, unavoidable), **course change** (a removal or a reversed
decision), **traverse** (many platforms or targets), **side line** (a
tool or library beside the main one), **recovery** (fixes what the last
one broke), **candidate** (the release before a major).

### Annapurna, 8,091 m (0.x and 1.x)

First ascent 3 June 1950 by Maurice Herzog and Louis Lachenal, French
expedition, by the North Face; the first 8000-metre peak climbed, on the
first attempt, without a prior reconnaissance of the mountain. Both
summiters lost fingers and toes on the descent.

| place | what it was | use it for |
| --- | --- | --- |
| Miristi Khola | the approach gorge; weeks spent finding a way in | the first release of a line, or one that is mostly finding the way (used: 0.1.0) |
| Base Camp | where the expedition was staged | the release that sets up everything else: installers, CI, docs, process (used: 0.2.0) |
| Camp I to Camp V | the camps of the 1950 route; Camp V at 7,400 m was the summit-push camp | ordinary progress, each a stage higher; the last camp for the release the final push starts from (used: 0.3.0 to 0.7.0) |
| Cauliflower Ridge | the ice ridge tried first and abandoned for the Sickle route | course change: a removed feature, a reversed decision |
| the Sickle | the crescent glacier the summit route crosses; exposed, avalanche-prone | the last risky structural change before a major (used: 0.8.0) |
| Summit Ridge | the final ridge; nothing left but walking up | the last minor before a major (used: 0.9.0) |
| Summit | 3 June 1950 | `X.0.0` (used: 1.0.0) |
| North Face | the 1950 route, the face the camps are on | a release that completes the original line: finishes something begun in 0.x (used: 1.3.0) |
| Dutch Rib | the 1977 route on the North Face, now the usual line because it is safer than the Sickle | the release that makes the everyday path easier and safer: ergonomics (1.1.0) |
| South Face | Bonington's 1970 siege of the great wall; Whillans and Haston to the top; the first big-wall climb in the Himalaya | the biggest minor of a line, climbed the hard way: memory safety (1.2.0) |
| the Sanctuary | the glacial basin ringed by the Annapurna peaks; the base of every south-side route | the release that supplies everything else: the standard library (planned: 1.4.0) |
| East Ridge | the long traverse route over the massif's summits (1984) | traverse: many platforms from one source (planned: 1.5.0) |
| North-West Face | Messner and Kammerlander, 1985, fast and light | performance: fewer instructions, less memory, the same programs (planned: 1.6.0) |
| Annapurna II | the massif's second summit, 7,937 m, joined to the main one by a ridge that runs the length of the range | the release that joins Nexium to other languages: interop, the seam (planned: 1.7.0) |
| Gangapurna, Annapurna South, Hiunchuli, Annapurna III, IV | the massif's other summits | side line: the GUI, the registry, a tool beside the compiler (Gangapurna planned: 1.8.0) |
| Machapuchare | the fishtail peak at the mouth of the Sanctuary, sacred and unclimbed by agreement | a release that stops short on purpose: deprecations only, a feature freeze |
| Fang | the sharp neighbour of Annapurna South | hard crux: one small difficult change |
| Descent | the retreat through the monsoon after the summit, on frostbitten feet | recovery |
| Annapurna Circuit | the trek around the whole massif | a release that is all documentation and examples: the book, the tour |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Herzog | leader and summiter; wrote the book | follows a big release, shipped by whoever leads it (used: 0.2.1) |
| Lachenal | summiter; wanted to turn back and went on | finishes something against the odds (used: 0.6.1) |
| Terray | carried the frostbitten summiters down | rescues a broken release: the hotfix (used: 1.0.2) |
| Rébuffat | the guide; roped the snow-blind party together on the descent | ties loose ends: docs, tests, consistency (used: 1.0.1) |
| Couzy | the engineer of the team | fixes the build, the toolchain, the C |
| Schatz | found the summit party in the crevasse the morning after | finds a long-hidden bug (used: 1.0.3) |
| Oudot | the doctor; amputations on the march out | removes something to save the rest |
| Ichac | the filmmaker | is all documentation, examples, screenshots |
| de Noyelle | liaison officer; permits and diplomacy | packaging, licensing, distribution channels (used: 1.2.1) |
| Ang Tharkey | the sirdar; declined the summit and kept the camps supplied | CI and infrastructure, the work that carries everything else |
| Bonington, Whillans, Haston | 1970, the South Face | repairs the biggest feature of a line |
| Messner, Kammerlander | 1985, the North-West Face | makes something faster |
| Steck | 2013, the South Face alone in a day | a one-person, one-day fix that changes a lot |

### Everest, 8,849 m (2.x)

First ascent 29 May 1953 by Edmund Hillary and Tenzing Norgay, British
expedition led by John Hunt, by the South Col route.

| place | what it was | use it for |
| --- | --- | --- |
| Base Camp | 5,364 m, on the Khumbu glacier | groundwork right after the major |
| Khumbu Icefall | the moving icefall every south-side climb crosses first; dangerous and unglamorous | the release after a major that carries the migrations and the fallout (planned: 2.1.0) |
| Western Cwm | the silent glacier valley above the icefall | consolidation (planned: 2.2.0) |
| Camp I to Camp IV | the camps of the standard route | ordinary progress |
| Lhotse Face | the long steep ice slope | the grind: performance, hardening (planned: 2.3.0) |
| Geneva Spur, Yellow Band | the rock steps on the way to the col | hard crux |
| South Col | 7,950 m, the last camp, exposed to the wind | the staging release before the next big feature (planned: 2.4.0) |
| the Balcony | the ledge at 8,400 m where the summit day pauses | a small release that lets the next one breathe |
| South Summit | the false summit at 8,749 m | candidate |
| Hillary Step | the last rock step before the top | the last technical crux of the line (planned: 2.5.0) |
| Summit | 29 May 1953 | `X.0.0` |
| Rongbuk, North Col, Second Step, Norton Couloir | the north side, the pre-war British route | the other way up: a second backend, a second implementation |
| West Ridge, Hornbein Couloir | the 1963 American traverse, up one side and down the other | traverse |
| Kangshung Face | the east face, avalanche-swept, climbed 1983 | a big subsystem climbed the hard way |
| Nuptse, Lhotse | the neighbours that form the Cwm's walls | side line |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Hunt | leader; the plan was his | reorganizes the release process |
| Hillary | summiter | lands the missing piece of the headline feature |
| Tenzing | summiter, on his seventh Everest expedition | fixes a regression in something that used to work |
| Bourdillon, Evans | reached the South Summit three days earlier and turned back with failing oxygen sets | pulls a feature back at the last minute |
| Lowe | cut the route up the Lhotse Face | fixes the build |
| Westmacott | kept the Icefall route open all season | keeps the migration path working |
| Band, Noyce | carried to the South Col | groundwork |
| Wylie | organization and transport | packaging |
| Ward | doctor | correctness |
| Pugh | physiologist; oxygen, water, food | benchmarks and measurement |
| Stobart, Gregory, Morris | film, photographs, the news | documentation and announcements |
| Mallory, Irvine | lost near the top in 1924 | fixes a bug reported years ago |
| Norton | 8,570 m without oxygen, 1924 | gets far with no new dependency |
| Tabei | first woman on the summit, 1975 | a first of its kind |
| Messner, Habeler | without oxygen, 1978 | removes a dependency |
| Hornbein, Unsoeld | the West Ridge traverse, 1963 | crosses platforms |

### Nanga Parbat, 8,126 m (3.x)

First ascent 3 July 1953 by Hermann Buhl, alone on the summit day, without
oxygen, from a German-Austrian expedition led by Karl Herrligkoffer, by the
Rakhiot Face. Thirty-one people had died on earlier attempts.

| place | what it was | use it for |
| --- | --- | --- |
| Fairy Meadows | the base camp meadow under the Rakhiot Face | groundwork |
| Rakhiot Face, Rakhiot Peak | the 1953 route and the peak on its ridge | progress on the original line |
| Silver Saddle, Silver Plateau | the high col and the plateau before the summit pyramid | consolidation at altitude: everything in place, nothing yet used |
| Moor's Head | the rock outcrop on the ridge | hard crux |
| Bazhin Gap | the notch before the summit | candidate |
| Summit | 3 July 1953, Buhl alone | `X.0.0` |
| Diamir Face, Kinshofer Route | the 1962 route, now the usual line | the everyday route made safer: ergonomics |
| Rupal Face | the highest mountain face on Earth, 4,500 m; the Messner brothers, 1970 | the biggest minor of a line |
| Mazeno Ridge | the longest ridge on any 8000er, first traversed in 2012 | traverse |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Buhl | went on alone when told to retreat; bivouacked standing on a ledge | one person's fix that everyone else had given up on |
| Herrligkoffer | leader, who ordered the retreat | reverses a decision |
| Aschenbrenner | climbing leader, turned back below the Silver Saddle | stops short deliberately |
| Kempter | turned back on the summit day | a partial fix, finished later |
| Frauenberger, Rainer, Köllensperger | carried to the high camps | groundwork |
| Ertl | filmmaker | documentation |
| Mummery | the first attempt on any 8000er, 1895, lost on the mountain | fixes the oldest bug in the tracker |
| Kinshofer, Löw, Mannhardt | 1962, the Diamir Face | ergonomics |
| Reinhold and Günther Messner | 1970, the Rupal Face; Günther lost on the descent | a big fix with a cost |
| Moro, Txikon, Sadpara | the first winter ascent, 2016 | done in the worst conditions: a fix under deadline |

### K2, 8,611 m (4.x)

First ascent 31 July 1954 by Lino Lacedelli and Achille Compagnoni, Italian
expedition led by Ardito Desio, by the Abruzzi Spur. Walter Bonatti and the
Hunza porter Amir Mahdi carried the oxygen to 8,100 m and survived a night
in the open to do it.

| place | what it was | use it for |
| --- | --- | --- |
| Baltoro | the glacier of the approach | groundwork |
| Concordia | where the glaciers meet and the mountain first shows | the release where the shape of the line becomes clear |
| Base Camp | at the foot of the spur | groundwork |
| Abruzzi Spur | the standard route, the south-east ridge | progress on the original line |
| House's Chimney | the rock chimney at 6,600 m, the first crux | hard crux |
| Black Pyramid | the rock band above Camp II | the hard middle of a line |
| the Shoulder | the snow shoulder, the last camp | the staging release |
| the Bottleneck | the narrow couloir under the summit serac, the crux; narrow, hard, unavoidable | a small release with one difficult change everyone must pass |
| the Serac | the ice cliff over the Bottleneck | a risk that hangs over a release: a known issue shipped |
| Summit | 31 July 1954 | `X.0.0` |
| Cesen Route | the safer south-south-east spur | ergonomics |
| Magic Line | the south-south-west pillar, one of the hardest routes on any 8000er | the biggest minor of a line |
| North Ridge | the Chinese side | a second implementation |
| Savoia Glacier, Godwin-Austen Glacier | the glaciers either side | side line |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Desio | leader, a geologist | reorganizes |
| Lacedelli, Compagnoni | summiters | land the headline |
| Bonatti | carried the oxygen to 8,100 m and was left out of the summit | the fix nobody credits that made the release possible |
| Mahdi | the Hunza porter who carried with Bonatti and lost his toes | infrastructure with a cost |
| Puchoz | died at Camp II | removes something |
| Abram, Angelino, Floreanini, Gallotti, Rey, Soldà, Viotto | carried to the high camps | groundwork |
| Pagani | doctor | correctness |
| Fantin | filmmaker | documentation |
| Rutkiewicz | first woman, 1986 | a first |
| Diemberger, Tullis | the 1986 storm | recovery after a bad release |
| Purja, Mingma G | the first winter ascent, 2021 | done in the worst conditions |

### Cho Oyu, 8,188 m (5.x)

First ascent 19 October 1954 by Herbert Tichy, Joseph Jöchler and Pasang
Dawa Lama: a three-climber expedition, the first 8000er climbed after the
monsoon and the smallest team to climb one. Tichy's hands were frostbitten
before the summit push and he went anyway.

| place | what it was | use it for |
| --- | --- | --- |
| Nangpa La | the trade pass beside the mountain | a release that is mostly interop: crossing to another language or platform |
| the Ice Cliff | the serac band at 6,600 m, the route's one technical step | hard crux |
| North-West Ridge | the standard route | progress |
| Summit Plateau | the summit is a broad plateau; the top is hard to find | a release that is done but nobody is sure where the end is: consolidation |
| Summit | 19 October 1954 | `X.0.0` |
| Gokyo, Ngozumpa Glacier | the Nepal side | side line |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Tichy | leader and summiter, with frostbitten hands | ships despite the state of the tooling |
| Jöchler | summiter | lands the headline |
| Pasang Dawa Lama | went down for supplies and came back up in time to summit | catches up: a backport |
| Heuberger | the expedition's geographer | measurement, benchmarks |

### Makalu, 8,485 m (6.x)

First ascent 15 May 1955 by Lionel Terray and Jean Couzy, French expedition
led by Jean Franco; every member of the team reached the summit over the
following two days, the first time an 8000er's whole expedition did.

| place | what it was | use it for |
| --- | --- | --- |
| Barun Valley, Barun Glacier | the approach | groundwork |
| Makalu La | the col of the 1955 route | the staging release |
| French Couloir | the summit couloir | the last crux |
| Summit | 15 May 1955, then everyone | `X.0.0` |
| West Pillar | 1971, one of the great Himalayan rock routes | the biggest minor of a line |
| South-East Ridge | the long ridge | traverse |
| Kangchungtse, Chomo Lonzo | the neighbouring summits | side line |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Franco | leader; brought everyone to the top | a release where every contributor's change lands |
| Terray, Couzy | first to the summit, both Annapurna veterans | the fix by people who did it before |
| Magnone | Fitz Roy with Terray, 1952 | a hard rock problem: the parser, the grammar |
| Bouvier, Coupé, Leroux, Vialatte | the second and third summit days | follow-up patches that complete the set |
| Gyalzen Norbu | sirdar; summited here and on Manaslu | infrastructure that carries two projects |
| Moro, Urubko | the first winter ascent, 2009 | done in the worst conditions |

### Kangchenjunga, 8,586 m (7.x)

First ascent 25 May 1955 by Joe Brown and George Band, British expedition
led by Charles Evans, from the Yalung Glacier. They stopped a few steps
below the top, as promised to the ruler of Sikkim, and every ascent since
has done the same: the summit is left untrodden.

| place | what it was | use it for |
| --- | --- | --- |
| Yalung Glacier | the 1955 approach | groundwork |
| the Hump | the first rise of the route | early progress |
| Great Shelf | the wide snow terrace at 7,700 m | consolidation |
| the Gangway | the ramp to the summit ridge | the staging release |
| Summit | 25 May 1955, one step short by promise | `X.0.0`; and the mountain's tradition makes *Summit* here the release that keeps one promise unbroken: a major with no breaking change |
| Talung Saddle, Zemu Glacier | the east side | side line |
| North Col, North Ridge | Boardman, Tasker and Scott, 1979, without oxygen | a second way up, without the usual aids |
| the Pinnacles | the sharp ridge on the north-east | hard crux |
| Five Treasures | the mountain's name, its five summits | a release with five headline items |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Evans | leader; deputy on Everest two years before | reorganizes, from experience |
| Brown | the rock climber who led the final crack | the technical fix |
| Band | summiter, Everest 1953 veteran | lands the headline |
| Hardie, Streather | the second pair, next day | the follow-up that completes the set |
| Jackson, Mather, MacKinnon | carried to the high camps | groundwork |
| Clegg | doctor | correctness |
| Dawa Tenzing | sirdar | infrastructure |
| Boardman, Tasker, Scott | 1979, the North Ridge | removes a dependency |

### Manaslu, 8,163 m (8.x)

First ascent 9 May 1956 by Toshio Imanishi and Gyalzen Norbu, Japanese
expedition led by Yuko Maki; a second pair, Kiichiro Kato and Minoru
Higeta, followed two days later.

| place | what it was | use it for |
| --- | --- | --- |
| Samagaon | the village below the mountain | groundwork |
| Naike Col | the col of the route | early progress |
| the Plateau | the great snow terrace under the summit | consolidation |
| North-East Face | the standard route | progress |
| East Pinnacle | the sharp subsidiary summit | hard crux |
| Summit | 9 May 1956 | `X.0.0` |
| Larkya La | the pass of the circuit trek | traverse |
| Birendra Lake | the glacial lake under the base camp | a quiet release: documentation, examples |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Maki | leader, who had made the first ascent of the Eiger's Mittellegi Ridge in 1921 | reorganizes, from long experience |
| Imanishi | summiter | lands the headline |
| Gyalzen Norbu | summiter, sirdar, Makalu the year before | infrastructure |
| Kato, Higeta | the second pair | the follow-up |

### Lhotse, 8,516 m (9.x)

First ascent 18 May 1956 by Ernst Reiss and Fritz Luchsinger, Swiss
expedition led by Albert Eggler, which made the second and third ascents
of Everest the same week.

| place | what it was | use it for |
| --- | --- | --- |
| Lhotse Face | the ice slope shared with the Everest route | the grind |
| Reiss Couloir | the summit couloir | the last crux |
| Summit | 18 May 1956 | `X.0.0` |
| Lhotse Shar | the eastern summit | side line |
| Lhotse Middle | the last unclimbed 8000-metre summit, 2001 | the feature that stayed open longest, finally done |
| South Face | one of the great unsolved walls until 1990 | the biggest minor of a line |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Eggler | leader; two mountains in one expedition | a patch that serves two lines at once |
| Reiss, Luchsinger | summiters | land the headline |
| Schmied, Marmet | Everest's second ascent | repeat something that worked, on purpose |
| von Gunten, Reist | Everest's third ascent, the next day | the follow-up |

### Gasherbrum II, 8,035 m (10.x)

First ascent 7 July 1956 by Fritz Moravec, Josef Larch and Hans Willenpart,
Austrian expedition, after an open bivouac at 7,500 m.

| place | what it was | use it for |
| --- | --- | --- |
| Gasherbrum Icefall | the icefall of the approach | groundwork |
| Gasherbrum Cwm | the glacier basin between the Gasherbrums | consolidation |
| Banana Ridge | the curved south-west ridge of the standard route | progress with a bend in it: a release that changed direction halfway and still arrived |
| Gasherbrum La | the col to Gasherbrum I | traverse |
| Abruzzi Glacier | the glacier below | side line |
| Summit | 7 July 1956 | `X.0.0` |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Moravec, Larch, Willenpart | the three summiters, after a night out at 7,500 m | a fix that took a night nobody planned |
| Moro, Urubko, Richards | the first winter ascent of a Karakoram 8000er, 2011 | done in the worst conditions |

### Broad Peak, 8,051 m (11.x)

First ascent 9 June 1957 by Marcus Schmuck, Fritz Wintersteller, Kurt
Diemberger and Hermann Buhl: four climbers, no oxygen, no high-altitude
porters, the first 8000er climbed in what became alpine style. Buhl died
three weeks later on Chogolisa.

| place | what it was | use it for |
| --- | --- | --- |
| West Spur | the standard route | progress |
| the Col | the 7,800 m saddle between the summits | the staging release |
| Rocky Summit | the fore-summit, an hour short of the top, where many stop | a release that is almost there and says so |
| Central Summit, North Summit | the other tops of the massif | side line |
| Summit | 9 June 1957, all four | `X.0.0` |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Schmuck, Wintersteller | first to the top | land the headline |
| Diemberger | summited here and on Dhaulagiri, the only person with two 8000er first ascents | serves two lines |
| Buhl | Nanga Parbat and Broad Peak; lost on Chogolisa weeks later | the last fix from someone leaving the project |
| Bielecki, Małek | the first winter ascent, 2013 | done in the worst conditions |
| Berbeka, Kowalski | lost on the descent of that ascent | a fix with a cost |

### Gasherbrum I (Hidden Peak), 8,080 m (12.x)

First ascent 5 July 1958 by Pete Schoening and Andy Kauffman, American
expedition led by Nick Clinch. In 1975 Reinhold Messner and Peter Habeler
climbed it in pure alpine style, the first 8000er done that way.

| place | what it was | use it for |
| --- | --- | --- |
| Hidden Peak | its other name: invisible until the last bend of the glacier | a release whose size only shows at the end |
| Roch Arête | the 1958 route | progress on the original line |
| Japanese Couloir | the standard route now | ergonomics |
| Gasherbrum La | the col to Gasherbrum II | traverse |
| North-West Face | Messner and Habeler, 1975, alpine style | performance: the same thing with less |
| Summit | 5 July 1958 | `X.0.0` |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Clinch | leader | reorganizes |
| Schoening | summiter; the man of "the Belay" on K2 in 1953, who held five falling climbers on one ice axe | the one fix that stops a cascade of failures |
| Kauffman | summiter | lands the headline |
| Nevison | the expedition doctor | correctness |
| Messner, Habeler | 1975, alpine style | removes a dependency |

### Dhaulagiri, 8,167 m (13.x)

First ascent 13 May 1960 by Kurt Diemberger, Peter Diener, Ernst Forrer,
Albin Schelbert, Nyima Dorje and Nawang Dorje, Swiss-Austrian expedition
led by Max Eiselin, supplied by a Pilatus Porter aircraft that landed on the
mountain at 5,700 m and was wrecked there before the summit: the only
8000er climbed with an aeroplane. Herzog's 1950 expedition had reconnoitred
it first and given up, which is how they came to Annapurna.

| place | what it was | use it for |
| --- | --- | --- |
| Hidden Valley | the approach | groundwork |
| French Pass, Dhampus Pass | the passes of the approach | traverse |
| Dapa Col | where the aircraft landed | a release delivered by a new tool |
| North-East Ridge | the 1960 route, the standard line | progress |
| the Pear | the buttress of the 1980s routes | hard crux |
| South Face | still unclimbed direct | a release that is deliberately not attempted: a "won't do" written down |
| Summit | 13 May 1960 | `X.0.0` |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Eiselin | leader; bought the aircraft | reorganizes with new tooling |
| Diemberger | summiter, his second 8000er first ascent | serves two lines |
| Diener, Forrer, Schelbert | summiters | land the headline |
| Nyima Dorje, Nawang Dorje | summiters, Sherpas | infrastructure |
| Saxer, Wick | the pilots | the tool that got you far and broke: a build-tool patch |
| the Yeti | the aircraft | a patch to a tool, not a person |

### Shishapangma, 8,027 m (14.x)

First ascent 2 May 1964 by a Chinese expedition led by Xu Jing: ten
climbers on the summit, the last 8000er climbed, and the only one entirely
inside Tibet.

| place | what it was | use it for |
| --- | --- | --- |
| Nyalam | the road head | groundwork |
| North Face | the standard route | progress |
| Central Summit | 8,008 m, where most ascents end; the true top is a corniced ridge away | a release that claims less than it could: honest, incomplete |
| Main Summit | the true top | `X.0.0` |
| South-West Face | Scott, MacIntyre and Baxter-Jones, 1982, alpine style | the biggest minor of a line |
| Gosainthan | the mountain's Sanskrit name | a release that renames things |

| person | who they were | fits a patch that |
| --- | --- | --- |
| Xu Jing | leader | reorganizes |
| Wang Fuzhou, Zhang Junyan, Sodnam Doje, Migmar Trashi | summiters | land the headline, in order |
| Scott, MacIntyre, Baxter-Jones | 1982, the South-West Face | removes a dependency |

### After the 8000ers: the Seven Summits (15.x to 20.x)

The highest peak on each continent, in this order.

| mountain | first ascent | places | use them for |
| --- | --- | --- | --- |
| Aconcagua, 6,961 m | Zurbriggen, 1897 | Plaza de Mulas, Nido de Cóndores, the Canaleta, Polish Glacier, Summit | the Canaleta (the loose scree gully at the top) for the tedious last crux; Polish Glacier for the elegant hard way |
| Denali, 6,190 m | Stuck, Karstens, Harper, Tatum, 1913 | Kahiltna Glacier, Motorcycle Hill, Windy Corner, the Headwall, West Buttress, Denali Pass, Cassin Ridge, Summit | Windy Corner for a release exposed to the weather (upstream changes); Cassin Ridge for the biggest minor |
| Kilimanjaro, 5,895 m | Meyer, Purtscheller, 1889 | Machame, Barranco Wall, Barafu, Stella Point, Kibo, Uhuru, Summit | Stella Point for the candidate (the crater rim, an hour short); Uhuru ("freedom") for a release that removes a constraint |
| Elbrus, 5,642 m | east summit Khashirov, 1829; west summit Grove, Gardiner, Walker, Knubel, 1874 | the Barrels, Pastukhov Rocks, the Saddle, Summit | the Saddle for a release between two peaks (two headline features) |
| Vinson, 4,892 m | Clinch's expedition, 1966 | Branscomb Glacier, Low Camp, High Camp, Summit | Vinson for the line nobody visits: a maintenance major |
| Puncak Jaya, 4,884 m | Harrer, 1962 | Carstensz Pyramid, Summit; with Kosciuszko, 2,228 m, for the list's other definition | Kosciuszko for the easiest major there will ever be |

People: Zurbriggen; Stuck, Karstens, Harper (first on the summit), Tatum;
Meyer, Purtscheller; Khashirov; Grove, Gardiner, Walker, Knubel; Clinch;
Harrer. Everest is used by 2.x; Mont Blanc, 4,808 m (Paccard and Balmat,
1786), opens the Alps.

### After those: the great north faces of the Alps (21.x onward)

| mountain | first ascent of the face | places | use them for |
| --- | --- | --- | --- |
| Eiger | Heckmair, Vörg, Harrer, Kasparek, 1938 | Difficult Crack, Hinterstoisser Traverse, Swallow's Nest, the Flatiron, Death Bivouac, the Ramp, Traverse of the Gods, the White Spider, Exit Cracks, Summit | Hinterstoisser Traverse for a step you cannot reverse (a migration with no way back); Death Bivouac for the release that nearly ended the line; the White Spider for the exposed crux; Exit Cracks for the candidate |
| Matterhorn | Whymper, 1865; the north face by the Schmid brothers, 1931 | Hörnli Ridge, Solvay Hut, the Shoulder, Zmutt Ridge, Lion Ridge, Summit | Solvay Hut for an emergency release; Zmutt Ridge for the elegant alternative |
| Grandes Jorasses | Walker Spur by Cassin, Esposito, Tizzoni, 1938 | Croz Spur, Walker Spur, Pointe Walker, Summit | the Walker Spur for the biggest minor |
| Piz Badile | Cassin, 1937 | North-East Face, Summit | a clean hard line |
| Cima Grande di Lavaredo | Comici, 1933 | North Face, Summit | Comici's "drop of water" line: the release that does one thing straight |
| Petit Dru | Allain, Leininger, 1935; the Bonatti Pillar, solo, 1955 | Bonatti Pillar, Summit | one person's six-day solo: a large fix by one contributor |

People: Heckmair, Vörg, Harrer, Kasparek; Whymper; the Schmid brothers;
Cassin, Esposito, Tizzoni; Comici; Allain, Leininger; Bonatti.
