# CivTerm — Advances (Technology)

Reference for every researchable advancement: cost, prerequisites, and what it
unlocks. Prerequisites must all be discovered before an advance becomes
researchable. Costs are in research points; cities contribute beakers each
turn (boosted by Library/University).

## Founding advances (no prerequisites)

| Advance          | Cost | Unlocks (advances)                              | Units unlocked        | Improvements unlocked |
| ---------------- | ---- | ----------------------------------------------- | --------------------- | --------------------- |
| Alphabet         | 30   | Code of Laws, Map Making, Mathematics, Writing   | —                     | —                     |
| Bronze Working   | 30   | Currency, Iron Working                          | Phalanx, Trireme      | —                     |
| Ceremonial Burial| 30   | Mysticism                                      | —                     | Temple                |
| Horseback Riding | 30   | —                                              | Cavalry*              | —                     |
| Masonry          | 30   | Construction, Feudalism                        | —                     | City Walls, Palace    |
| Pottery          | 30   | —                                              | —                     | Granary               |
| Wheel            | 30   | Engineering                                    | Chariot, Catapult     | —                     |

\* Cavalry also requires Chivalry.

## Mid-tier advances

| Advance          | Cost | Prerequisites              | Unlocks (advances)              | Units unlocked | Improvements unlocked |
| ---------------- | ---- | -------------------------- | ------------------------------- | -------------- | --------------------- |
| Code of Laws     | 100  | Alphabet                   | Literacy, Trade                 | —              | Courthouse            |
| Currency         | 80   | Bronze Working             | Construction, Trade             | —              | —                     |
| Iron Working     | 80   | Bronze Working             | Bridge Building                 | Legion         | —                     |
| Map Making       | 100  | Alphabet                   | Navigation                      | Sail           | —                     |
| Mathematics      | 100  | Alphabet, Masonry          | Astronomy, University           | —              | —                     |
| Mysticism        | 80   | Ceremonial Burial          | Astronomy, Philosophy           | —              | —                     |
| Writing          | 140  | Alphabet                   | Literacy, Religion              | Diplomat       | Library               |
| Construction     | 20   | Masonry, Currency          | Engineering                     | —              | Aqueduct, Colosseum   |
| Feudalism        | 160  | Masonry                    | Chivalry                        | —              | —                     |
| Bridge Building  | 300  | Iron Working, Alphabet     | —                               | —              | —                     |
| Engineering      | 280  | Wheel, Construction        | Invention                       | —              | —                     |
| Literacy         | 360  | Writing, Code of Laws      | Invention, Philosophy           | —              | —                     |
| Trade            | 320  | Currency, Code of Laws     | Banking, Medicine               | Caravan        | —                     |
| University       | 480  | Mathematics, Philosophy    | Chemistry, Theory of Gravity    | —              | University            |
| Philosophy       | 320  | Mysticism, Literacy        | Medicine, Religion, University  | —              | —                     |

## Late-tier advances

| Advance          | Cost | Prerequisites              | Unlocks (advances)              | Units unlocked | Improvements unlocked |
| ---------------- | ---- | -------------------------- | ------------------------------- | -------------- | --------------------- |
| Astronomy        | 360  | Mysticism, Mathematics     | Navigation, Theory of Gravity   | —              | —                     |
| Banking          | 320  | Trade                      | —                               | —              | Bank                  |
| Chivalry         | 400  | Feudalism, Horseback Riding| —                               | Knight         | —                     |
| Medicine         | 640  | Philosophy, Trade          | Chemistry                       | —              | —                     |
| Navigation       | 600  | Map Making, Astronomy      | Magnetism, Physics              | —              | —                     |
| Physics          | 520  | Mathematics, Navigation    | Magnetism                       | —              | —                     |
| Religion         | 600  | Philosophy, Writing        | —                               | —              | Cathedral             |
| Chemistry        | 600  | University, Medicine       | Refining                        | —              | —                     |
| Invention        | 640  | Engineering, Literacy      | —                               | —              | —                     |
| Magnetism        | 720  | Navigation, Physics        | —                               | Frigate        | —                     |
| Refining         | 720  | Chemistry                  | —                               | —              | —                     |
| Theory of Gravity| 800  | Astronomy, University      | —                               | —              | —                     |

## Unlock summary

### Units
| Unit     | Requires         |
| -------- | ---------------- |
| Phalanx  | Bronze Working   |
| Legion   | Iron Working     |
| Chariot  | Wheel            |
| Cavalry  | Horseback Riding |
| Knight   | Chivalry         |
| Catapult | Wheel            |
| Diplomat | Writing          |
| Caravan  | Trade            |
| Trireme  | Bronze Working   |
| Sail     | Map Making       |
| Frigate  | Magnetism        |

### Improvements
| Improvement | Requires          |
| ----------- | ----------------- |
| Temple      | Ceremonial Burial |
| Granary     | Pottery           |
| City Walls  | Masonry           |
| Palace      | Masonry           |
| Aqueduct    | Construction      |
| Colosseum   | Construction      |
| Library     | Writing           |
| Courthouse  | Code of Laws      |
| Marketplace | Currency          |
| Bank        | Banking           |
| Cathedral   | Religion          |
| University  | University        |

## Research mechanics

- Each city contributes research points per turn based on population
  (diminishing returns), boosted by Library (×5/4) and University (×3/2).
- The player chooses a research target; discovery grants a dialog to pick
  the next.
- Rivals research automatically (scaled by difficulty).
- Diplomats can steal technology from rivals.

## Notes

- Founding advances (Alphabet, Bronze Working, Ceremonial Burial,
  Horseback Riding, Masonry, Pottery, Wheel) have no prerequisites and are
  researchable from the start.
- `Advancement::prerequisites()` is the single source of the tree;
  `Advancement::cost()` is the single source of research costs.