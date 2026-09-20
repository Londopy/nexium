# Release names

Every Nexium release is a place on a mountain. A major version is a mountain,
taken in the order the fourteen 8000-metre peaks were first climbed; the
versions under it are the climb of that mountain: its camps, routes and faces
for minor versions, the members of its first-ascent expedition for patch
versions, and `Summit` for `X.0.0`. The 0.x line is the approach and the
camps of the first mountain, so 1.0.0 is standing on top of the mountain the
project has been on since 0.1.0. Decision 89 in `DECISIONS.md` records the
rule; this file is the reference and the ledger.

## How to read a name

```
nx 0.7.0 (Annapurna: Camp V)
nx 1.0.0 (Annapurna: Summit)
nx 1.1.0 (Annapurna: South Face)
nx 2.0.0 (Everest: Summit)
nx 2.1.0 (Everest: Khumbu Icefall)
```

The mountain says which major line you are on; the place says how far up it
is. A version number still carries the ordering; the name carries the
character of the release.

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
  feature. Chosen to fit what the release did, with one line of why in the
  changelog. Short names: `Camp V`, not `the Sickle Glacier Couloir`.
- **Patch = a person from the mountain's first-ascent expedition**, in
  roster order, so no metaphor is forced onto a bug-fix release. When the
  roster runs out, later notable climbers of that mountain follow.
- **0.x** are the approach and the camps of the first climb; if there are
  more minor releases than camps, the features of the summit route follow
  (the Sickle, the Summit Ridge).
- The name fits the release; the release is never shaped to fit a name.

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
- `nx version`: `nx 0.7.0 (Annapurna: Camp V)`; the name is `RELEASE_NAME`
  in `src/main.rs`, set by the release commit with the version.

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
| 0.8.0 | Annapurna: the Sickle | reserved: the glacier couloir the summit route crosses |
| 0.9.0 | Annapurna: Summit Ridge | reserved |
| 1.0.0 | Annapurna: Summit | reserved |

## The pools

Names to draw from, mountain by mountain, in the order the mountains are
used. Places are for minor versions, people for patches. Each entry has a
gloss so a fitting name can be picked without a history book; the histories
are the standard accounts, and a wrong detail here is fixed in this file.

### Annapurna, 8,091 m (0.x and 1.x)

First ascent 3 June 1950 by Maurice Herzog and Louis Lachenal, French
expedition, by the North Face; the first 8000-metre peak climbed, on the
first attempt, without a prior reconnaissance of the mountain. Both
summiters lost fingers and toes on the descent.

Places:

- **Miristi Khola** — the gorge on the approach; weeks were spent finding a
  way in (used: 0.1.0).
- **Base Camp** (used: 0.2.0), **Camp I** to **Camp V** (used: 0.3.0 to
  0.7.0; Camp V at 7,400 m was the summit-push camp).
- **the Sickle** — the crescent-shaped glacier below the summit that the
  1950 route crosses (reserved: 0.8.0).
- **Summit Ridge** (reserved: 0.9.0), **Summit** (reserved: 1.0.0).
- **North Face** — the 1950 route; the face the camps are on.
- **Cauliflower Ridge** — the ice ridge the 1950 team first tried before
  finding the Sickle route; a good name for a release that changed course.
- **South Face** — Chris Bonington's 1970 expedition (Don Whillans and
  Dougal Haston to the top): the first big-wall climb in the Himalaya.
- **Dutch Rib** — the 1977 route on the North Face, now the usual line.
- **North-West Face** — Reinhold Messner and Hans Kammerlander, 1985.
- **East Ridge** — the long traverse route (Swiss, 1984).
- **the Sanctuary** — the glacial basin ringed by the Annapurna peaks; the
  base camp of the South Face.
- **Annapurna South**, **Gangapurna**, **Hiunchuli**, **Annapurna II, III,
  IV** — the massif's other summits, for releases beside the main line
  (a tool, a library).
- **Machapuchare** — the fishtail peak at the mouth of the Sanctuary,
  sacred and unclimbed: a release that stops just short on purpose (a
  deprecation, a feature freeze).
- **Descent** — the retreat through the monsoon after the summit; for a
  release that is all recovery.

People (the 1950 expedition, in roster order):

1. **Herzog** — Maurice Herzog, leader and summiter (used: 0.2.1)
2. **Lachenal** — Louis Lachenal, summiter (used: 0.6.1)
3. **Terray** — Lionel Terray, who carried the summit pair down
4. **Rébuffat** — Gaston Rébuffat
5. **Couzy** — Jean Couzy
6. **Schatz** — Marcel Schatz
7. **Oudot** — Jacques Oudot, the expedition doctor
8. **Ichac** — Marcel Ichac, the filmmaker
9. **de Noyelle** — Francis de Noyelle, liaison officer
10. **Ang Tharkey** — the sirdar

Then the later climbers: Bonington, Whillans, Haston (1970); Messner,
Kammerlander (1985); Loretan, Kukuczka; Steck (2013 South Face).

### Everest, 8,849 m (2.x)

First ascent 29 May 1953 by Edmund Hillary and Tenzing Norgay, British
expedition led by John Hunt, by the South Col route.

Places: **Base Camp**, **Khumbu Icefall** (the dangerous, unglamorous
part every climb crosses first: groundwork), **Western Cwm** (the silent
valley above the icefall), **Camp I** to **Camp IV**, **Lhotse Face**,
**Geneva Spur**, **Yellow Band**, **South Col** (the last camp, at 7,950 m),
**the Balcony**, **South Summit**, **Hillary Step**, **Summit**; the north
side: **Rongbuk**, **North Col**, **the Second Step**, **Norton Couloir**;
**West Ridge** and **Hornbein Couloir** (the 1963 American traverse);
**Kangshung Face** (the east face, climbed 1983); **Nuptse** and **Lhotse**
for releases beside the main line.

People (1953): **Hunt** (leader), **Hillary**, **Tenzing**, **Bourdillon**,
**Evans** (the pair that reached the South Summit first), **Band**, **Noyce**,
**Lowe**, **Westmacott**, **Wylie**, **Ward** (doctor), **Pugh**
(physiologist), **Stobart** (filmmaker), **Gregory** (photographer),
**Morris** (the correspondent who got the news to London for the
coronation). Then **Mallory** and **Irvine** (1924), **Norton** (8,570 m
without oxygen, 1924), **Tabei** (first woman, 1975), **Messner** and
**Habeler** (no oxygen, 1978), **Hornbein** and **Unsoeld** (West Ridge,
1963).

### Nanga Parbat, 8,126 m (3.x)

First ascent 3 July 1953 by Hermann Buhl, alone on the summit day, without
oxygen, from a German-Austrian expedition led by Karl Herrligkoffer, by the
Rakhiot Face. The mountain had killed thirty-one people on earlier attempts.

Places: **Rakhiot Face** (the 1953 route), **Rakhiot Peak**, **Silver
Saddle** (Silbersattel), **Silver Plateau**, **Moor's Head** (Mohrenkopf),
**Bazhin Gap**, **Diamir Face** and the **Kinshofer Route** (1962, now the
usual line), **Rupal Face** (the highest mountain face on Earth, 4,500 m;
Reinhold and Günther Messner, 1970), **Mazeno Ridge** (the longest ridge on
any 8000er, first traversed 2012), **Fairy Meadows** (the base camp
meadow), **Summit**.

People (1953): **Buhl**, **Herrligkoffer** (leader), **Aschenbrenner**
(climbing leader), **Kempter**, **Frauenberger**, **Rainer**,
**Köllensperger**, **Ertl** (filmmaker). Then **Mummery** (1895, the first
attempt on any 8000er, lost on the mountain), **Kinshofer**, **Löw**,
**Mannhardt** (1962), the **Messner** brothers (1970), **Moro**, **Txikon**,
**Sadpara** (the first winter ascent, 2016).

### K2, 8,611 m (4.x)

First ascent 31 July 1954 by Lino Lacedelli and Achille Compagnoni, Italian
expedition led by Ardito Desio, by the Abruzzi Spur. Walter Bonatti and the
Hunza porter Amir Mahdi carried the oxygen to 8,100 m and survived a night
in the open without a tent to do it.

Places: **Baltoro** (the glacier of the approach), **Concordia** (where the
glaciers meet and the mountain first shows), **Base Camp**, **Abruzzi
Spur** (the standard route), **House's Chimney**, **Black Pyramid**, **the
Shoulder** (the last camp), **the Bottleneck** (the couloir under the
summit serac, the crux: a narrow, hard release), **the Serac**, **Summit**;
**Cesen Route**, **Magic Line** (the south-south-west pillar), **North
Ridge**, **Savoia Glacier**, **Godwin-Austen Glacier**.

People (1954): **Desio** (leader), **Lacedelli**, **Compagnoni**,
**Bonatti**, **Mahdi**, **Abram**, **Angelino**, **Floreanini**,
**Gallotti**, **Puchoz** (died on the expedition), **Rey**, **Soldà**,
**Viotto**, **Pagani** (doctor), **Fantin** (filmmaker). Then
**Rutkiewicz** (first woman, 1986), **Diemberger**, **Tullis** (1986),
**Purja** and **Mingma G** (the first winter ascent, 2021).

### Cho Oyu, 8,188 m (5.x)

First ascent 19 October 1954 by Herbert Tichy, Joseph Jöchler and Pasang
Dawa Lama: a three-climber expedition, the first 8000er climbed after the
monsoon and the smallest team to climb one. Tichy's hands were frostbitten
before the summit push and he went anyway.

Places: **Nangpa La** (the trade pass beside the mountain), **the Ice
Cliff** (the serac band at 6,600 m, the route's one technical step),
**North-West Ridge** (the standard route), **Gokyo** and the **Ngozumpa
Glacier** (the Nepal side), **Summit Plateau** (the summit is a broad
plateau and hard to find), **Summit**.

People (1954): **Tichy**, **Jöchler**, **Pasang Dawa Lama**, **Heuberger**
(the expedition's geographer).

### Makalu, 8,485 m (6.x)

First ascent 15 May 1955 by Lionel Terray and Jean Couzy, French expedition
led by Jean Franco; every member of the team reached the summit over the
following two days, the first time an 8000er's whole expedition did.

Places: **Barun Valley** and **Barun Glacier** (the approach), **Makalu La**
(the col of the 1955 route), **French Couloir** (the summit couloir),
**West Pillar** (1971), **South-East Ridge**, **Kangchungtse** (Makalu II)
and **Chomo Lonzo** (the neighbouring summits), **Summit**.

People (1955): **Franco** (leader), **Terray**, **Couzy**, **Magnone**,
**Bouvier**, **Coupé**, **Leroux**, **Vialatte**, **Gyalzen Norbu**
(sirdar). Then **Moro** and **Urubko** (the first winter ascent, 2009).

### Kangchenjunga, 8,586 m (7.x)

First ascent 25 May 1955 by Joe Brown and George Band, British expedition
led by Charles Evans, by the south-west face from the Yalung Glacier. They
stopped a few steps below the top, as promised to the ruler of Sikkim, and
every ascent since has done the same: the summit is left untrodden.

Places: **Yalung Glacier** (the 1955 approach), **the Hump**, **Great
Shelf** (the wide snow terrace of the route), **the Gangway** (the ramp to
the summit ridge), **Talung Saddle**, **Zemu Glacier** (the east side),
**North Col** and **North Ridge** (Boardman, Tasker and Scott, 1979,
without oxygen), **the Pinnacles**, **Five Treasures** (the mountain's name:
its five summits), **Summit** — by tradition the release that stops one
step short: a release candidate.

People (1955): **Evans** (leader), **Brown**, **Band**, **Hardie**,
**Streather** (the second summit pair), **Jackson**, **Mather**,
**MacKinnon**, **Clegg** (doctor), **Dawa Tenzing** (sirdar). Then
**Boardman**, **Tasker**, **Scott** (1979).

### Manaslu, 8,163 m (8.x)

First ascent 9 May 1956 by Toshio Imanishi and Gyalzen Norbu, Japanese
expedition led by Yuko Maki; a second pair, Kiichiro Kato and Minoru
Higeta, followed two days later.

Places: **Samagaon** (the village below the mountain), **Naike Col**, **the
Plateau** (the great snow terrace under the summit), **North-East Face**
(the standard route), **East Pinnacle**, **Larkya La** (the pass of the
circuit trek), **Birendra Lake**, **Summit**.

People (1956): **Maki** (leader), **Imanishi**, **Gyalzen Norbu**, **Kato**,
**Higeta**.

### Lhotse, 8,516 m (9.x)

First ascent 18 May 1956 by Ernst Reiss and Fritz Luchsinger, Swiss
expedition led by Albert Eggler, which made the second ascent of Everest
the same week.

Places: **Lhotse Face** (shared with the Everest route), **Reiss Couloir**
(the summit couloir), **Lhotse Shar** and **Lhotse Middle** (the last
unclimbed 8000-metre summit, 2001), **South Face** (one of the great
unsolved walls until 1990), **Summit**.

People (1956): **Eggler** (leader), **Reiss**, **Luchsinger**, **Schmied**
and **Marmet** (Everest's second ascent), **von Gunten**, **Reist**.

### Gasherbrum II, 8,035 m (10.x)

First ascent 7 July 1956 by Fritz Moravec, Josef Larch and Hans Willenpart,
Austrian expedition, after an open bivouac at 7,500 m.

Places: **Gasherbrum Icefall**, **Gasherbrum Cwm**, **Banana Ridge** (the
curved south-west ridge of the standard route), **Gasherbrum La**,
**Abruzzi Glacier**, **Summit**.

People (1956): **Moravec**, **Larch**, **Willenpart**. Then **Moro**,
**Urubko**, **Richards** (the first winter ascent of any Karakoram 8000er,
2011).

### Broad Peak, 8,051 m (11.x)

First ascent 9 June 1957 by Marcus Schmuck, Fritz Wintersteller, Kurt
Diemberger and Hermann Buhl: four climbers, no oxygen, no high-altitude
porters, the first 8000er climbed in what became alpine style. Buhl died
three weeks later on Chogolisa.

Places: **West Spur** (the standard route), **the Col** (the 7,800 m saddle
between the summits), **Rocky Summit** (the fore-summit where many stop,
an hour short of the top: a release that is almost there), **Central
Summit**, **North Summit**, **Summit**.

People (1957): **Schmuck**, **Wintersteller**, **Diemberger**, **Buhl**.
Then **Bielecki**, **Małek**, **Berbeka**, **Kowalski** (the first winter
ascent, 2013, which cost the last two their lives).

### Gasherbrum I (Hidden Peak), 8,080 m (12.x)

First ascent 5 July 1958 by Pete Schoening and Andy Kauffman, American
expedition led by Nick Clinch. In 1975 Reinhold Messner and Peter Habeler
climbed it in pure alpine style, the first 8000er done that way.

Places: **Hidden Peak** (its other name: the mountain is invisible until
the last bend of the glacier — a release whose size only shows at the end),
**Roch Arête** (the 1958 route), **Japanese Couloir** (the standard route
now), **Gasherbrum La**, **North-West Face** (Messner and Habeler, 1975),
**Summit**.

People (1958): **Clinch** (leader), **Schoening**, **Kauffman**,
**Nevison**. Then **Messner**, **Habeler** (1975).

### Dhaulagiri, 8,167 m (13.x)

First ascent 13 May 1960 by Kurt Diemberger, Peter Diener, Ernst Forrer,
Albin Schelbert, Nyima Dorje and Nawang Dorje, Swiss-Austrian expedition
led by Max Eiselin, supplied by a Pilatus Porter aircraft that landed on the
mountain at 5,700 m — the only 8000er climbed with an aeroplane. Herzog's
1950 expedition had reconnoitred it first and given up, which is how they
came to Annapurna.

Places: **Hidden Valley** (the approach), **French Pass** and **Dhampus
Pass**, **the Dapa Col** (where the aircraft landed), **North-East Ridge**
(the 1960 route and standard line), **the Pear** (the buttress of the 1980s
routes), **South Face** (still unclimbed direct: a release that is not
attempted), **Summit**.

People (1960): **Eiselin** (leader), **Diemberger**, **Diener**, **Forrer**,
**Schelbert**, **Nyima Dorje**, **Nawang Dorje**, **Saxer** and **Wick**
(the pilots), **the Yeti** (the aircraft; for a patch that is a tool, not a
person).

### Shishapangma, 8,027 m (14.x)

First ascent 2 May 1964 by a Chinese expedition led by Xu Jing: ten
climbers on the summit, the last 8000er climbed, and the only one entirely
inside Tibet.

Places: **Nyalam** (the road head), **North Face** (the standard route),
**Central Summit** (8,008 m, where most ascents end; the true summit is a
corniced ridge away — a release that claims less than it could), **Main
Summit** (**Summit**), **South-West Face** (Scott, MacIntyre and
Baxter-Jones, 1982, alpine style), **Gosainthan** (the mountain's Sanskrit
name).

People (1964): **Xu Jing** (leader), **Wang Fuzhou**, **Zhang Junyan**,
**Sodnam Doje**, **Migmar Trashi**. Then **Scott**, **MacIntyre**,
**Baxter-Jones** (1982).

### After the 8000ers: the Seven Summits (15.x to 20.x)

The highest peak on each continent, in this order:

- **Aconcagua**, 6,961 m (South America; Matthias Zurbriggen, 1897):
  Plaza de Mulas, Nido de Cóndores, the Canaleta, Polish Glacier, Summit.
- **Denali**, 6,190 m (North America; Stuck, Karstens, Harper and Tatum,
  1913): Kahiltna Glacier, Motorcycle Hill, Windy Corner, the Headwall,
  West Buttress, Denali Pass, Cassin Ridge, Summit.
- **Kilimanjaro**, 5,895 m (Africa; Meyer and Purtscheller, 1889): Machame,
  Barranco Wall, Barafu, Stella Point, Kibo, Uhuru, Summit.
- **Elbrus**, 5,642 m (Europe; the east summit by Killar Khashirov, 1829,
  the west by Grove, Gardiner, Walker and Knubel, 1874): the Barrels,
  Pastukhov Rocks, the Saddle, Summit.
- **Vinson**, 4,892 m (Antarctica; Nicholas Clinch's expedition, 1966):
  Branscomb Glacier, Low Camp, High Camp, Summit.
- **Puncak Jaya** (Carstensz Pyramid), 4,884 m (Oceania; Heinrich Harrer,
  1962), with **Kosciuszko**, 2,228 m, for the list's other definition.

Everest is already used; Mont Blanc, 4,808 m (Paccard and Balmat, 1786),
opens the Alps.

### After those: the great north faces of the Alps (21.x onward)

- **Eiger** (Heckmair, Vörg, Harrer and Kasparek, 1938): Difficult Crack,
  Hinterstoisser Traverse, Swallow's Nest, the Flatiron, Death Bivouac, the
  Ramp, Traverse of the Gods, the White Spider, Exit Cracks, Summit.
- **Matterhorn** (Whymper, 1865; the north face by the Schmid brothers,
  1931): Hörnli Ridge, Solvay Hut, the Shoulder, Zmutt Ridge, Lion Ridge,
  Summit.
- **Grandes Jorasses** (Walker Spur by Cassin, Esposito and Tizzoni, 1938):
  Croz Spur, Walker Spur, Pointe Walker, Summit.
- **Piz Badile** (Cassin, 1937), **Cima Grande di Lavaredo** (Comici, 1933),
  **Petit Dru** (Allain and Leininger, 1935; the Bonatti Pillar, solo,
  1955).
