#!/usr/bin/env python3
"""The repository's assets, drawn from one description of the mark.

The mark is Fuji over a lake. The lake reflects the mountain, and the
reflection becomes the letters `nx` as it goes down. By day it is indigo on
washi; by night the same scene under a dark sky. Writes:

    assets/logo.svg                 the mark; night under prefers-color-scheme: dark
    assets/banner-light.svg         the README's banner, for <picture>
    assets/banner-dark.svg
    assets/social-preview.svg/.png  1280 x 640, the GitHub social preview
    assets/icon-256.png             the app icon, and
    assets/icon.ico                 the same at 16 to 256
    assets/icon-circle-1024.png     the scene composed for a circle (Discord, avatars)
    assets/icon-small.svg           the mark for tiny places (a browser tab, a window's
    assets/icon-small.ico, -64.png  title bar): the peak, the water and the sun, no letters
    editors/vscode/icon.png
    installers/windows/wizard-large.bmp, wizard-small.bmp

Needs Pillow and nothing else: the letters (Noto Serif Bold, under the SIL
Open Font License) are outlines in this file, and the rasters are drawn by
the small path rasterizer below, since Pillow reads no SVG.

    python scripts/make_assets.py            # everything
    python scripts/make_assets.py icon       # one group: logo, banner, social, icon, circle, small, installer
"""
import math
import os
import re
import sys

from PIL import Image, ImageChops, ImageDraw

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# ---- the two palettes ---------------------------------------------------------

DAY = dict(paper="#F6F1E7", skywash="#E3ECF1", cloud="#FFFFFF", mountain="#35557A", keyline="#2A4463",
           snow="#FBF9F4", water="#D5E3EC", deep="#C8D9E4", line="#FFFFFF", reflection="#35557A",
           letters="#2A4463", disc="#D9684B", disc_op=0.85, wordmark="#35557A")
NIGHT = dict(paper="#1B2A3C", skywash="#142030", cloud="#2A3D52", mountain="#4A6684", keyline="#3A536D",
             snow="#E6ECF1", water="#22384B", deep="#1B2E3F", line="#8FB0C8", reflection="#C9D8E4",
             letters="#E3ECF3", disc="#F2E9D8", disc_op=0.95, wordmark="#E6ECF1")

# ---- the letters: Noto Serif Bold, 1000 units per em, y down -----------------

NX = (
      "M20 0V-53H26Q59 -53 80 -65Q100 -77 100 -122V-418Q100 -460 81 -472Q62 -483 29 -483H25V-536H236L249 -465H254"
      "Q277 -506 312 -528Q348 -549 407 -549Q486 -549 528 -503Q571 -457 571 -356V-124Q571 -77 587 -65"
      "Q603 -53 637 -53H641V0H419V-329Q419 -393 402 -428Q386 -464 342 -464Q308 -464 288 -442Q269 -421 261 -386"
      "Q253 -350 253 -309V-118Q253 -76 272 -64Q290 -53 323 -53H327V0ZM678 0V-53H687Q723 -53 744 -67"
      "Q765 -81 795 -117L922 -269L796 -430Q774 -456 753 -470Q732 -483 710 -483H697V-536H1008V-483H1004"
      "Q972 -483 962 -474Q951 -466 951 -454Q951 -436 974 -408L1030 -340L1075 -397Q1086 -413 1094 -426"
      "Q1102 -439 1102 -453Q1102 -471 1088 -477Q1075 -483 1054 -483H1050V-536H1279V-483H1270Q1241 -483 1222 -470"
      "Q1202 -457 1170 -418L1066 -293L1213 -106Q1236 -78 1255 -66Q1274 -53 1293 -53H1306V0H988V-53H993"
      "Q1053 -53 1053 -86Q1053 -97 1046 -111Q1039 -125 1015 -155L958 -224L892 -137Q883 -126 875 -112"
      "Q867 -98 867 -85Q867 -69 881 -61Q895 -53 928 -53H932V0Z",
      1313)
NEXIUM = (
      "M28 0V-53H53Q83 -53 102 -64Q121 -75 121 -118V-600Q121 -640 102 -650Q83 -661 58 -661H28V-714H251L603 -216"
      "V-600Q603 -637 585 -649Q567 -661 541 -661H511V-714H772V-661H741Q714 -661 696 -648Q678 -634 678 -596V0H586"
      "L196 -551V-118Q196 -75 212 -64Q229 -53 259 -53H289V0ZM1094 10Q966 10 900 -62Q835 -135 835 -265"
      "Q835 -406 900 -478Q964 -549 1083 -549Q1192 -549 1254 -488Q1317 -427 1317 -308V-257H989Q992 -157 1026 -111"
      "Q1061 -65 1129 -65Q1180 -65 1213 -90Q1246 -114 1263 -148Q1277 -144 1287 -132Q1297 -121 1297 -104"
      "Q1297 -78 1276 -52Q1255 -26 1210 -8Q1166 10 1094 10ZM1164 -321Q1164 -397 1146 -440Q1128 -484 1085 -484"
      "Q1043 -484 1018 -442Q994 -401 991 -321ZM1371 0V-53H1380Q1416 -53 1437 -67Q1458 -81 1488 -117L1615 -269"
      "L1489 -430Q1467 -456 1446 -470Q1425 -483 1403 -483H1390V-536H1701V-483H1697Q1665 -483 1654 -474"
      "Q1644 -466 1644 -454Q1644 -436 1667 -408L1723 -340L1768 -397Q1779 -413 1787 -426Q1795 -439 1795 -453"
      "Q1795 -471 1782 -477Q1768 -483 1747 -483H1743V-536H1972V-483H1963Q1934 -483 1914 -470Q1895 -457 1863 -418"
      "L1759 -293L1906 -106Q1929 -78 1948 -66Q1967 -53 1986 -53H1999V0H1681V-53H1686Q1746 -53 1746 -86"
      "Q1746 -97 1739 -111Q1732 -125 1708 -155L1651 -224L1585 -137Q1576 -126 1568 -112Q1560 -98 1560 -85"
      "Q1560 -69 1574 -61Q1588 -53 1621 -53H1625V0ZM2177 -626Q2141 -626 2116 -644Q2092 -661 2092 -698"
      "Q2092 -736 2116 -753Q2141 -770 2177 -770Q2212 -770 2238 -753Q2263 -736 2263 -698Q2263 -661 2238 -644"
      "Q2212 -626 2177 -626ZM2023 0V-53H2035Q2065 -53 2086 -67Q2106 -81 2106 -124V-416Q2106 -456 2086 -470"
      "Q2065 -483 2035 -483H2023V-536H2258V-124Q2258 -81 2278 -67Q2299 -53 2329 -53H2341V0ZM2622 10"
      "Q2535 10 2494 -38Q2454 -87 2454 -188V-412Q2454 -455 2439 -469Q2424 -483 2387 -483H2384V-536H2606V-216"
      "Q2606 -152 2622 -114Q2638 -75 2682 -75Q2730 -75 2752 -116Q2773 -157 2773 -227V-419Q2773 -462 2754 -472"
      "Q2736 -483 2707 -483H2704V-536H2925V-116Q2925 -73 2944 -63Q2963 -53 2992 -53H3000V0H2801L2779 -71H2774"
      "Q2748 -28 2710 -9Q2673 10 2622 10ZM3045 0V-53H3053Q3086 -53 3106 -65Q3125 -77 3125 -122V-421"
      "Q3125 -463 3106 -474Q3086 -486 3053 -486H3050V-536H3260L3273 -465H3278Q3302 -507 3338 -528"
      "Q3375 -549 3437 -549Q3490 -549 3527 -529Q3564 -509 3582 -465H3588Q3633 -549 3747 -549Q3827 -549 3871 -503"
      "Q3915 -457 3915 -356V-124Q3915 -77 3932 -65Q3948 -53 3982 -53H3986V0H3763V-329Q3763 -393 3745 -428"
      "Q3727 -464 3683 -464Q3652 -464 3632 -444Q3613 -425 3604 -392Q3596 -359 3596 -321V-124Q3596 -77 3612 -65"
      "Q3629 -53 3663 -53H3666V0H3444V-329Q3444 -393 3427 -428Q3410 -464 3366 -464Q3333 -464 3314 -442"
      "Q3294 -421 3286 -386Q3277 -350 3277 -309V-118Q3277 -76 3296 -64Q3316 -53 3349 -53H3352V0Z",
      4011)

# the prose on the social card: Noto Serif Regular, the glyphs it needs

SERIF_SPACE = 260
SERIF_KERN = {"We": -60, "py": -20, "ra": -20, "y,": -90, "y.": -90}
SERIF = {
    ',': (
          "M30 112Q125 81 125 22Q125 7 114 -1Q103 -9 88 -16Q74 -23 63 -35Q52 -47 52 -70Q52 -99 70 -114"
          "Q89 -129 117 -129Q148 -129 171 -106Q194 -84 194 -42Q194 22 156 75Q119 128 30 154Z",
        250),
    '.': (
          "M125 7Q99 7 80 -8Q62 -23 62 -61Q62 -100 80 -114Q99 -129 125 -129Q151 -129 170 -114Q188 -100 188 -61"
          "Q188 -23 170 -8Q151 7 125 7Z",
        250),
    '/': (
          "M0 121 230 -760H288L59 121Z",
        288),
    'A': (
          "M0 0V-42H19Q48 -42 62 -57Q77 -72 95 -120L317 -714H395L621 -95Q632 -64 648 -53Q663 -42 692 -42H705V0"
          "H430V-42H453Q513 -42 513 -90Q513 -98 511 -107Q509 -116 505 -127L465 -239H202L164 -134"
          "Q155 -110 155 -91Q155 -42 221 -42H244V0ZM221 -289H447L385 -464Q369 -508 356 -547Q343 -586 335 -622"
          "Q328 -586 317 -553Q306 -520 289 -473Z",
        705),
    'C': (
          "M361 10Q262 10 194 -36Q126 -82 92 -164Q57 -247 57 -358Q57 -466 94 -548Q130 -631 202 -678"
          "Q273 -724 378 -724Q480 -724 530 -692Q580 -660 580 -612Q580 -580 555 -561Q530 -542 493 -542"
          "Q493 -573 482 -602Q471 -632 446 -652Q420 -671 376 -671Q263 -671 216 -590Q168 -508 168 -358"
          "Q168 -269 190 -200Q211 -132 257 -94Q303 -56 376 -56Q449 -56 490 -82Q532 -107 557 -141"
          "Q565 -136 570 -126Q576 -117 576 -102Q576 -77 553 -51Q530 -25 482 -8Q435 10 361 10Z",
        614),
    'N': (
          "M38 0V-42H51Q85 -42 109 -54Q133 -67 133 -114V-604Q133 -648 108 -660Q84 -672 51 -672H38V-714H221"
          "L579 -161V-604Q579 -648 554 -660Q530 -672 497 -672H484V-714H735V-672H722Q688 -672 664 -660"
          "Q640 -647 640 -600V0H569L194 -576V-114Q194 -67 218 -54Q242 -42 276 -42H289V0Z",
        763),
    'P': (
          "M38 0V-42H51Q85 -42 109 -54Q133 -67 133 -114V-604Q133 -648 108 -660Q84 -672 51 -672H38V-714H319"
          "Q445 -714 505 -658Q565 -603 565 -505Q565 -446 540 -394Q514 -342 455 -310Q396 -278 297 -278H234V-109"
          "Q234 -65 258 -54Q283 -42 316 -42H349V0ZM287 -325Q378 -325 418 -366Q457 -406 457 -501"
          "Q457 -585 422 -626Q387 -666 302 -666H234V-325Z",
        604),
    'R': (
          "M38 0V-42H51Q84 -42 108 -54Q133 -65 133 -109V-604Q133 -648 108 -660Q84 -672 51 -672H38V-714H307"
          "Q564 -714 564 -521Q564 -468 542 -432Q520 -395 486 -373Q453 -351 417 -339L554 -122Q580 -82 604 -62"
          "Q628 -42 659 -42H662V0H648Q586 0 552 -6Q517 -13 496 -32Q475 -52 452 -90L317 -315H234V-109"
          "Q234 -65 258 -54Q283 -42 316 -42H329V0ZM304 -362Q392 -362 424 -401Q457 -440 457 -518"
          "Q457 -598 422 -632Q387 -666 302 -666H234V-362Z",
        656),
    'S': (
          "M247 10Q149 10 98 -29Q48 -68 48 -131Q48 -161 67 -180Q86 -198 120 -198Q122 -157 136 -120"
          "Q150 -84 179 -62Q208 -39 255 -39Q322 -39 360 -72Q398 -104 398 -165Q398 -202 384 -228"
          "Q369 -253 336 -275Q302 -297 243 -321Q150 -359 104 -410Q59 -461 59 -543Q59 -600 88 -640"
          "Q116 -681 166 -702Q215 -724 278 -724Q367 -724 416 -690Q465 -656 465 -612Q465 -580 444 -564"
          "Q422 -547 386 -547Q386 -578 376 -607Q365 -636 340 -655Q316 -674 274 -674Q216 -674 184 -643"
          "Q152 -612 152 -560Q152 -520 166 -492Q181 -465 215 -444Q249 -422 307 -398Q395 -362 444 -315"
          "Q493 -268 493 -191Q493 -96 426 -43Q360 10 247 10Z",
        544),
    'W': (
          "M93 -619Q83 -651 67 -662Q51 -672 22 -672H9V-714H284V-672H261Q201 -672 201 -624Q201 -616 204 -607"
          "Q206 -598 209 -587L301 -267Q314 -222 325 -178Q336 -134 344 -98Q352 -137 362 -184Q372 -230 385 -278"
          "L504 -707H576L701 -274Q715 -225 726 -179Q737 -133 744 -98Q752 -134 762 -174Q771 -214 784 -262"
          "L872 -571Q875 -583 879 -599Q883 -615 883 -623Q883 -672 817 -672H794V-714H1038V-672H1019"
          "Q990 -672 974 -658Q957 -644 943 -594L774 0H684L521 -560L368 0H276Z",
        1047),
    'a': (
          "M205 10Q138 10 94 -29Q50 -68 50 -150Q50 -230 106 -268Q163 -306 278 -310L361 -313V-373"
          "Q361 -409 355 -436Q349 -464 329 -480Q309 -496 268 -496Q230 -496 210 -482Q190 -468 184 -444"
          "Q177 -419 177 -387Q135 -387 114 -402Q92 -416 92 -450Q92 -485 116 -506Q141 -527 182 -536"
          "Q223 -546 272 -546Q364 -546 410 -507Q455 -468 455 -373V-114Q455 -72 469 -57Q483 -42 517 -42H520V0"
          "H385L369 -86H361Q340 -58 320 -36Q300 -15 274 -2Q247 10 205 10ZM228 -52Q289 -52 325 -90"
          "Q361 -127 361 -191V-272L297 -269Q212 -265 180 -234Q147 -204 147 -145Q147 -52 228 -52Z",
        563),
    'b': (
          "M355 10Q297 10 260 -14Q224 -39 202 -78H196L178 0H18V-42H26Q60 -42 84 -54Q108 -67 108 -114V-650"
          "Q108 -694 84 -706Q59 -718 26 -718H18V-760H202V-576Q202 -559 202 -532Q201 -506 200 -482"
          "Q199 -457 198 -446H202Q225 -492 261 -519Q297 -546 355 -546Q454 -546 506 -480Q559 -413 559 -269"
          "Q559 -124 506 -57Q454 10 355 10ZM339 -54Q405 -54 434 -110Q462 -165 462 -270Q462 -377 434 -430"
          "Q405 -482 338 -482Q260 -482 231 -430Q202 -377 202 -269Q202 -165 231 -110Q260 -54 339 -54Z",
        614),
    'c': (
          "M283 10Q217 10 166 -18Q114 -46 84 -106Q55 -167 55 -265Q55 -372 84 -434Q114 -495 164 -520"
          "Q215 -546 278 -546Q320 -546 360 -535Q400 -524 426 -502Q452 -479 452 -444Q452 -410 430 -396"
          "Q407 -381 363 -381Q363 -428 346 -462Q328 -496 278 -496Q240 -496 212 -476Q183 -456 168 -406"
          "Q152 -356 152 -266Q152 -160 188 -108Q223 -55 303 -55Q350 -55 384 -74Q419 -94 436 -125"
          "Q453 -111 453 -86Q453 -63 434 -41Q415 -19 378 -4Q340 10 283 10Z",
        492),
    'd': (
          "M259 10Q160 10 108 -56Q55 -123 55 -267Q55 -412 108 -479Q160 -546 259 -546Q317 -546 354 -522"
          "Q390 -497 412 -458H418Q415 -483 414 -514Q412 -544 412 -568V-650Q412 -694 388 -706Q363 -718 330 -718"
          "H322V-760H506V-110Q506 -66 530 -54Q555 -42 588 -42H596V0H427L416 -90H412Q390 -44 354 -17"
          "Q318 10 259 10ZM276 -54Q354 -54 383 -106Q412 -159 412 -267Q412 -371 383 -426Q354 -482 275 -482"
          "Q209 -482 180 -426Q152 -371 152 -266Q152 -160 180 -107Q209 -54 276 -54Z",
        614),
    'e': (
          "M287 10Q178 10 116 -62Q55 -134 55 -264Q55 -404 113 -475Q171 -546 277 -546Q374 -546 430 -486"
          "Q485 -426 485 -307V-261H152Q154 -152 192 -102Q229 -53 301 -53Q353 -53 390 -74Q426 -96 444 -123"
          "Q451 -120 457 -111Q463 -102 463 -89Q463 -69 444 -46Q425 -23 386 -6Q347 10 287 10ZM384 -315"
          "Q384 -395 360 -444Q335 -492 275 -492Q220 -492 190 -446Q159 -401 154 -315Z",
        535),
    'f': (
          "M27 0V-42H40Q74 -42 98 -54Q122 -67 122 -114V-489H31V-536H122V-586Q122 -675 170 -722"
          "Q217 -770 299 -770Q377 -770 408 -750Q439 -731 439 -700Q439 -673 416 -658Q393 -644 357 -644"
          "Q357 -674 343 -699Q329 -724 291 -724Q248 -724 232 -691Q216 -658 216 -595V-536H357V-489H216V-114"
          "Q216 -67 240 -54Q264 -42 298 -42H336V0Z",
        369),
    'g': (
          "M231 240Q127 240 75 202Q23 163 23 94Q23 35 61 5Q99 -25 148 -34Q128 -43 110 -64Q93 -84 93 -116"
          "Q93 -146 108 -168Q124 -190 158 -210Q115 -228 92 -270Q68 -311 68 -361Q68 -447 115 -496"
          "Q162 -546 256 -546Q292 -546 324 -536Q356 -526 370 -513Q384 -529 409 -548Q434 -567 467 -567"
          "Q497 -567 512 -552Q526 -536 526 -515Q526 -494 514 -478Q501 -463 473 -463Q473 -474 466 -486"
          "Q460 -497 440 -497Q417 -497 397 -485Q414 -464 425 -436Q436 -407 436 -364Q436 -290 392 -241"
          "Q347 -192 256 -192Q244 -192 228 -194Q213 -195 203 -197Q184 -187 170 -172Q156 -157 156 -134"
          "Q156 -116 168 -106Q179 -96 218 -96H331Q420 -96 458 -54Q496 -12 496 53Q496 139 432 190"
          "Q367 240 231 240ZM253 -240Q302 -240 322 -270Q342 -300 342 -365Q342 -433 322 -465Q301 -497 252 -497"
          "Q204 -497 183 -464Q162 -431 162 -364Q162 -300 184 -270Q205 -240 253 -240ZM233 191Q305 191 344 176"
          "Q383 160 398 132Q414 105 414 70Q414 24 388 8Q362 -7 312 -7H214Q186 -7 161 0Q136 8 120 28"
          "Q104 48 104 88Q104 117 115 140Q126 164 154 178Q182 191 233 191Z",
        538),
    'h': (
          "M18 0V-42H26Q60 -42 84 -54Q108 -67 108 -114V-650Q108 -694 84 -706Q59 -718 26 -718H18V-760H202V-540"
          "Q202 -522 201 -502Q200 -483 199 -469Q198 -455 198 -455H203Q249 -546 350 -546Q436 -546 482 -500"
          "Q527 -453 527 -350V-114Q527 -67 548 -54Q570 -42 604 -42H607V0H433V-345Q433 -410 408 -446"
          "Q384 -482 323 -482Q261 -482 232 -438Q202 -394 202 -320V-109Q202 -65 226 -54Q251 -42 284 -42H287V0Z",
        635),
    'i': (
          "M161 -636Q137 -636 120 -650Q104 -664 104 -698Q104 -733 120 -746Q137 -760 161 -760Q184 -760 201 -746"
          "Q218 -733 218 -698Q218 -664 201 -650Q184 -636 161 -636ZM23 0V-42H36Q69 -42 94 -54Q118 -65 118 -109"
          "V-426Q118 -470 94 -482Q69 -494 36 -494H33V-536H212V-114Q212 -67 236 -54Q260 -42 294 -42H307V0Z",
        320),
    'k': (
          "M18 0V-42H26Q60 -42 84 -54Q108 -67 108 -114V-650Q108 -694 84 -706Q59 -718 26 -718H18V-760H202V-374"
          "Q202 -361 202 -340Q201 -319 200 -298Q199 -277 198 -262Q198 -248 198 -248L323 -385Q355 -421 366 -440"
          "Q378 -460 378 -474Q378 -487 366 -490Q354 -494 329 -494V-536H550V-494Q516 -494 482 -470"
          "Q448 -445 409 -401L339 -322L472 -124Q498 -84 524 -63Q549 -42 583 -42H586V0H572Q529 0 500 -3"
          "Q471 -6 452 -16Q432 -25 414 -44Q397 -64 376 -97L276 -254L202 -199V-109Q202 -65 226 -54"
          "Q251 -42 284 -42H287V0Z",
        585),
    'l': (
          "M13 0V-42H26Q60 -42 84 -54Q108 -67 108 -114V-650Q108 -694 84 -706Q59 -718 26 -718H13V-760H202V-114"
          "Q202 -67 226 -54Q250 -42 284 -42H297V0Z",
        310),
    'm': (
          "M28 0V-42H41Q75 -42 96 -54Q118 -67 118 -114V-426Q118 -470 96 -482Q74 -494 41 -494H38V-536H195"
          "L208 -455H213Q243 -511 280 -528Q318 -546 364 -546Q412 -546 451 -526Q490 -505 509 -455H517"
          "Q547 -511 588 -528Q628 -546 674 -546Q751 -546 794 -500Q837 -453 837 -350V-114Q837 -67 858 -54"
          "Q880 -42 914 -42H917V0H743V-345Q743 -410 720 -446Q696 -482 638 -482Q597 -482 572 -462"
          "Q548 -441 538 -407Q527 -373 527 -333V-114Q527 -67 548 -54Q570 -42 604 -42H607V0H433V-345"
          "Q433 -410 410 -446Q386 -482 328 -482Q285 -482 260 -460Q234 -437 223 -400Q212 -363 212 -320V-109"
          "Q212 -65 236 -54Q261 -42 294 -42H297V0Z",
        945),
    'n': (
          "M28 0V-42H36Q70 -42 94 -54Q118 -67 118 -114V-426Q118 -470 94 -482Q71 -494 38 -494H33V-536H195"
          "L208 -455H213Q244 -511 282 -528Q321 -546 369 -546Q448 -546 492 -500Q537 -453 537 -350V-114"
          "Q537 -67 558 -54Q578 -42 612 -42H617V0H443V-345Q443 -410 418 -446Q394 -482 333 -482"
          "Q288 -482 262 -460Q235 -437 224 -400Q212 -363 212 -320V-109Q212 -65 236 -54Q259 -42 292 -42H297V0Z",
        645),
    'o': (
          "M287 10Q179 10 117 -59Q55 -128 55 -269Q55 -409 114 -478Q174 -546 290 -546Q398 -546 460 -478"
          "Q522 -409 522 -269Q522 -128 462 -59Q403 10 287 10ZM289 -42Q364 -42 394 -100Q425 -157 425 -269"
          "Q425 -381 394 -437Q363 -493 288 -493Q213 -493 182 -437Q152 -381 152 -269Q152 -157 183 -100"
          "Q214 -42 289 -42Z",
        577),
    'p': (
          "M18 240V198H26Q60 198 84 186Q108 173 108 126V-426Q108 -470 84 -482Q59 -494 26 -494H13V-536H188"
          "L198 -446H202Q225 -492 261 -519Q297 -546 355 -546Q454 -546 506 -480Q559 -413 559 -269"
          "Q559 -124 506 -57Q454 10 355 10Q297 10 260 -14Q224 -39 202 -78H198Q200 -49 201 -16Q202 16 202 35V131"
          "Q202 175 226 186Q251 198 284 198H287V240ZM339 -54Q405 -54 434 -110Q462 -165 462 -270"
          "Q462 -377 434 -430Q405 -482 338 -482Q260 -482 231 -430Q202 -377 202 -269Q202 -165 231 -110"
          "Q260 -54 339 -54Z",
        614),
    'r': (
          "M33 0V-42H36Q70 -42 94 -54Q118 -67 118 -114V-426Q118 -470 94 -482Q69 -494 36 -494H33V-536H187"
          "L206 -437H211Q224 -467 239 -492Q254 -517 279 -532Q304 -546 348 -546Q403 -546 430 -527"
          "Q456 -508 456 -473Q456 -442 434 -422Q413 -402 363 -402Q363 -443 351 -462Q339 -480 310 -480"
          "Q282 -480 263 -458Q244 -436 233 -402Q222 -368 217 -332Q212 -295 212 -266V-109Q212 -65 236 -54"
          "Q261 -42 294 -42H322V0Z",
        471),
    's': (
          "M210 10Q135 10 90 -17Q45 -44 45 -96Q45 -123 56 -138Q67 -153 82 -159Q96 -165 108 -165"
          "Q108 -113 132 -76Q155 -38 216 -38Q269 -38 298 -64Q326 -89 326 -129Q326 -154 316 -170"
          "Q305 -186 278 -202Q252 -217 203 -238Q152 -261 118 -282Q85 -304 68 -332Q52 -361 52 -404"
          "Q52 -472 104 -508Q155 -545 240 -545Q312 -545 349 -518Q386 -491 386 -453Q386 -426 368 -410"
          "Q349 -393 314 -393Q314 -443 293 -471Q272 -499 228 -499Q177 -499 155 -476Q133 -454 133 -419"
          "Q133 -381 162 -360Q190 -340 257 -313Q310 -291 343 -269Q376 -247 392 -218Q407 -189 407 -147"
          "Q407 -69 353 -30Q299 10 210 10Z",
        451),
    't': (
          "M240 10Q164 10 130 -24Q95 -59 95 -145V-479H19V-519Q37 -519 59 -526Q81 -534 97 -551Q114 -569 125 -595"
          "Q136 -621 143 -659H189V-536H320V-479H189V-142Q189 -91 210 -67Q231 -43 265 -43Q283 -43 298 -45"
          "Q313 -47 329 -50V-6Q316 0 290 5Q264 10 240 10Z",
        352),
    'u': (
          "M273 10Q194 10 151 -36Q108 -83 108 -186V-426Q108 -470 84 -482Q59 -494 26 -494H23V-536H202V-191"
          "Q202 -126 224 -90Q246 -54 307 -54Q373 -54 403 -98Q433 -143 433 -216V-422Q433 -469 409 -482"
          "Q385 -494 351 -494H348V-536H527V-109Q527 -65 552 -54Q576 -42 609 -42H612V0H453L440 -81H435"
          "Q404 -25 363 -8Q322 10 273 10Z",
        635),
    'v': (
          "M78 -441Q67 -473 52 -484Q36 -494 4 -494V-536H254V-494H241Q181 -494 181 -446Q181 -438 183 -429"
          "Q185 -420 189 -409L257 -220Q271 -183 284 -140Q297 -96 304 -70Q309 -91 325 -131Q341 -171 354 -207"
          "L426 -402Q435 -426 435 -445Q435 -494 369 -494H362V-536H576V-494H564Q535 -494 520 -479"
          "Q506 -464 488 -416L330 0H239Z",
        579),
    'w': (
          "M75 -441Q64 -473 48 -484Q33 -494 4 -494H1V-536H248V-494H235Q205 -494 190 -486Q175 -477 175 -452"
          "Q175 -444 178 -432Q180 -420 183 -409L230 -241Q237 -217 244 -188Q250 -158 256 -132Q262 -105 265 -88"
          "H268Q273 -113 284 -154Q296 -194 309 -231L413 -533H476L576 -237Q584 -213 593 -184Q602 -156 609 -130"
          "Q616 -105 619 -88H622Q631 -139 661 -234L711 -395Q715 -408 718 -422Q720 -437 720 -445"
          "Q720 -494 654 -494H647V-536H861V-494H848Q819 -494 804 -480Q788 -466 773 -416L644 0H564L427 -418"
          "L285 0H206Z",
        862),
    'x': (
          "M5 0V-42H14Q50 -42 72 -56Q94 -69 123 -106L249 -269L123 -441Q101 -467 80 -480Q60 -494 37 -494H24V-536"
          "H278V-494H275Q241 -494 230 -486Q218 -477 218 -465Q218 -455 224 -445Q230 -435 241 -419L307 -329"
          "L359 -404Q370 -421 377 -436Q384 -452 384 -465Q384 -483 368 -488Q353 -494 330 -494H327V-536H546V-494"
          "H537Q508 -494 487 -482Q466 -471 436 -430L333 -293L480 -95Q502 -66 522 -54Q541 -42 560 -42H573V0H315"
          "V-42H320Q380 -42 380 -75Q380 -86 372 -100Q365 -113 342 -144L275 -234L205 -136Q196 -124 187 -106"
          "Q178 -89 178 -73Q178 -57 192 -50Q207 -42 240 -42H243V0Z",
        578),
    'y': (
          "M38 193Q106 193 150 167Q193 141 220 96Q247 52 263 -4L78 -441Q65 -472 50 -483Q36 -494 7 -494H4V-536"
          "H244V-494H241Q181 -494 181 -446Q181 -429 189 -409L262 -231Q272 -208 282 -180Q291 -153 299 -128"
          "Q307 -103 310 -86Q317 -115 328 -146Q340 -177 350 -207L417 -402Q426 -426 426 -445Q426 -494 360 -494"
          "H357V-536H565V-494H562Q533 -494 518 -479Q503 -464 486 -416L334 4Q308 77 284 123Q260 169 230 194"
          "Q201 220 157 230Q113 240 47 240H38Z",
        565),
}

# ---- paths: parse, transform, write, flatten ---------------------------------

_TOKENS = re.compile(r"[MLHVCQZ]|-?\d*\.?\d+(?:e-?\d+)?")


def parse(d):
    """Absolute M, L, H, V, C, Q and Z to a list of commands with every point
    absolute: ('M', x, y), ('L', x, y), ('Q', x1, y1, x, y), ('C', ...), ('Z',)."""
    toks = _TOKENS.findall(d)
    out = []
    i = 0
    cx = cy = sx = sy = 0.0
    last = None
    while i < len(toks):
        c = toks[i]
        if c in "MLHVCQZ":
            i += 1
        else:
            # numbers where a command was expected repeat the last one
            # (after an M, as L), as SVG allows
            c = "L" if last == "M" else last
        last = c
        if c == "M":
            cx, cy = float(toks[i]), float(toks[i + 1])
            sx, sy = cx, cy
            out.append(("M", cx, cy))
            i += 2
        elif c == "L":
            cx, cy = float(toks[i]), float(toks[i + 1])
            out.append(("L", cx, cy))
            i += 2
        elif c == "H":
            cx = float(toks[i])
            out.append(("L", cx, cy))
            i += 1
        elif c == "V":
            cy = float(toks[i])
            out.append(("L", cx, cy))
            i += 1
        elif c == "Q":
            out.append(("Q", float(toks[i]), float(toks[i + 1]), float(toks[i + 2]), float(toks[i + 3])))
            cx, cy = float(toks[i + 2]), float(toks[i + 3])
            i += 4
        elif c == "C":
            out.append(("C", *[float(t) for t in toks[i:i + 6]]))
            cx, cy = float(toks[i + 4]), float(toks[i + 5])
            i += 6
        elif c == "Z":
            out.append(("Z",))
            cx, cy = sx, sy
        else:
            raise ValueError(c)
    return out


def mapped(cmds, fn):
    """The same path with every point put through fn(x, y) -> (x, y)."""
    out = []
    for c in cmds:
        if c[0] == "Z":
            out.append(c)
        else:
            pts = [fn(c[k], c[k + 1]) for k in range(1, len(c), 2)]
            out.append((c[0], *[v for p in pts for v in p]))
    return out


def write(cmds):
    parts = []
    for c in cmds:
        if c[0] == "Z":
            parts.append("Z")
        else:
            parts.append(c[0] + " ".join(f"{v:.1f}".rstrip("0").rstrip(".") for v in c[1:]))
    return "".join(parts)


def flatten(cmds, steps=12):
    """Polylines, each a list of (x, y), with whether it was closed."""
    polys = []
    cur = []
    closed = False
    cx = cy = 0.0

    def finish():
        nonlocal cur, closed
        if len(cur) > 1:
            polys.append((cur, closed))
        cur, closed = [], False

    for c in cmds:
        if c[0] == "M":
            finish()
            cx, cy = c[1], c[2]
            cur = [(cx, cy)]
        elif c[0] == "L":
            cx, cy = c[1], c[2]
            cur.append((cx, cy))
        elif c[0] == "Q":
            x1, y1, x, y = c[1:]
            for k in range(1, steps + 1):
                t = k / steps
                cur.append(((1 - t) ** 2 * cx + 2 * (1 - t) * t * x1 + t * t * x,
                            (1 - t) ** 2 * cy + 2 * (1 - t) * t * y1 + t * t * y))
            cx, cy = x, y
        elif c[0] == "C":
            x1, y1, x2, y2, x, y = c[1:]
            for k in range(1, steps + 1):
                t = k / steps
                u = 1 - t
                cur.append((u ** 3 * cx + 3 * u * u * t * x1 + 3 * u * t * t * x2 + t ** 3 * x,
                            u ** 3 * cy + 3 * u * u * t * y1 + 3 * u * t * t * y2 + t ** 3 * y))
            cx, cy = x, y
        elif c[0] == "Z":
            closed = True
            finish()
    finish()
    return polys


# ---- the scene ------------------------------------------------------------------

class El:
    """One shape: a fill or a stroke, a colour, an opacity, and a clip path."""

    def __init__(self, kind, cmds, color, opacity=1.0, width=0.0, clip=None):
        self.kind, self.cmds, self.color, self.opacity, self.width, self.clip = kind, cmds, color, opacity, width, clip


def rect(x, y, w, h, rx=0):
    if rx:
        k = 0.5523 * rx
        return parse(f"M{x + rx} {y}L{x + w - rx} {y}C{x + w - rx + k} {y} {x + w} {y + rx - k} {x + w} {y + rx}"
                     f"L{x + w} {y + h - rx}C{x + w} {y + h - rx + k} {x + w - rx + k} {y + h} {x + w - rx} {y + h}"
                     f"L{x + rx} {y + h}C{x + rx - k} {y + h} {x} {y + h - rx + k} {x} {y + h - rx}"
                     f"L{x} {y + rx}C{x} {y + rx - k} {x + rx - k} {y} {x + rx} {y}Z")
    return parse(f"M{x} {y}L{x + w} {y}L{x + w} {y + h}L{x} {y + h}Z")


def circle(cx, cy, r):
    k = 0.5523 * r
    return parse(f"M{cx + r} {cy}C{cx + r} {cy + k} {cx + k} {cy + r} {cx} {cy + r}C{cx - k} {cy + r} {cx - r} {cy + k} {cx - r} {cy}"
                 f"C{cx - r} {cy - k} {cx - k} {cy - r} {cx} {cy - r}C{cx + k} {cy - r} {cx + r} {cy - k} {cx + r} {cy}Z")


def fuji(bx0, bx1, cx, base, top):
    """Fuji: long concave flanks that flatten at the foot, a wide flat crater."""
    h = base - top
    return parse(f"M{bx0} {base}C{cx - 0.79 * h} {base - 0.1 * h} {cx - 0.25 * h} {top + 0.21 * h} {cx - 0.096 * h} {top}"
                 f"L{cx + 0.096 * h} {top}C{cx + 0.25 * h} {top + 0.21 * h} {cx + 0.79 * h} {base - 0.1 * h} {bx1} {base}Z")


def cap(cx, top, h):
    """The snow: one soft curve, lower in the middle, as it lies in the crater's lee."""
    u = h / 104
    return parse(f"M{cx - 10 * u} {top}L{cx + 10 * u} {top}C{cx + 16 * u} {top + 8 * u} {cx + 24 * u} {top + 15 * u} {cx + 30 * u} {top + 20 * u}"
                 f"C{cx + 16 * u} {top + 40 * u} {cx - 12 * u} {top + 42 * u} {cx - 30 * u} {top + 20 * u}"
                 f"C{cx - 24 * u} {top + 15 * u} {cx - 16 * u} {top + 8 * u} {cx - 10 * u} {top}Z")


def wave(x0, x1, y, amp, period, phase=0):
    """A wavy line from x0 to x1 about y, as path commands."""
    d = f"M{x0} {y}"
    x = x0
    up = phase % 2 == 0
    while x < x1 - 0.01:
        nx_ = min(x + period / 2, x1)
        d += f"Q{(x + nx_) / 2} {y + (-amp if up else amp)} {nx_} {y}"
        x = nx_
        up = not up
    return parse(d)


def region_above(x0, x1, y, amp, period, ytop):
    """The region from ytop down to a wavy edge at y."""
    return wave(x0, x1, y, amp, period) + [("L", x1, ytop), ("L", x0, ytop), ("Z",)]


def region_below(x0, x1, y, amp, period, ybot):
    return wave(x0, x1, y, amp, period) + [("L", x1, ybot), ("L", x0, ybot), ("Z",)]


def letters(glyphs, size, x, baseline):
    d, adv = glyphs
    k = size / 1000
    return mapped(parse(d), lambda px, py: (x + px * k, baseline + py * k)), adv * k


def prose(text, size, x, baseline):
    """A line of the regular serif, kerned, as path commands; and its width."""
    k = size / 1000
    cmds = []
    pos = 0.0
    prev = None
    for ch in text:
        if ch == " ":
            pos += SERIF_SPACE
            prev = None
            continue
        if prev is not None:
            pos += SERIF_KERN.get(prev + ch, 0)
        d, adv = SERIF[ch]
        cmds += mapped(parse(d), lambda px, py, pos=pos: (x + (px + pos) * k, baseline + py * k))
        pos += adv
        prev = ch
    return cmds, pos * k


def scene(P, W, H, horizon, cx, bw, s, sun=None, wordmark=None):
    """The mark at a size: a W x H picture, the horizon at `horizon`, the
    mountain's foot `bw` wide about `cx`, everything else scaled by `s`
    (1.0 is the 256-pixel square). sun: (x, y) or None. wordmark: (size,
    right margin, baseline) or None. Returns (background colour, elements)."""
    els = []
    top = horizon - 104 * s
    # the sky's top, in two steps, and the cloud strips of the prints
    els.append(El("fill", rect(0, 0, W, 22 * s), P["skywash"]))
    els.append(El("fill", rect(0, 22 * s, W, 18 * s), P["skywash"], 0.55))
    for (dx, dy, w, h) in ((-122, -64, 80, 9), (-96, -53, 30, 6), (36, -42, 52, 7)):
        els.append(El("fill", rect(cx + dx * s, horizon + dy * s, w * s, h * s, h * s / 2), P["cloud"]))
    if W > bw + 220 * s:
        els.append(El("fill", rect(W - 150 * s, horizon - 48 * s, 90 * s, 7 * s, 3.5 * s), P["cloud"]))
    if sun:
        els.append(El("fill", circle(sun[0], sun[1], 17 * s), P["disc"], P["disc_op"]))
    # the mountain, with its keyline, and the snow
    bx0, bx1 = cx - bw / 2, cx + bw / 2
    peak = fuji(bx0, bx1, cx, horizon, top)
    els.append(El("fill", peak, P["mountain"]))
    els.append(El("stroke", peak, P["keyline"], width=2 * s))
    els.append(El("fill", cap(cx, top, horizon - top), P["snow"]))
    # the lake
    els.append(El("fill", rect(0, horizon, W, H - horizon), P["water"]))
    els.append(El("fill", rect(0, horizon + 60 * s, W, H - horizon - 60 * s), P["deep"]))
    # the reflection: the flanks mirrored and compressed, down to a wavy
    # edge; the letters from a higher wavy edge down; between them, both
    y1, y2 = horizon + 26 * s, horizon + 50 * s
    amp, period = 3 * s, 40 * s
    inverted = mapped(peak, lambda x, y: (x, horizon + (horizon - y) * 0.7))
    els.append(El("fill", inverted, P["reflection"], 0.5, clip=region_above(-4, W + 4, y2, amp, period, horizon - 1)))
    nx, adv = letters(NX, 120 * s, 0, 0)
    nx, adv = letters(NX, 120 * s, cx - adv / 2, horizon + 90 * s)
    els.append(El("fill", nx, P["letters"], 0.9, clip=region_below(-4, W + 4, y1, amp, period, H + 4)))
    # the water lines: wavy, two weights, clear of the letters
    box = (cx - adv / 2 - 6 * s, cx + adv / 2 + 6 * s)
    rows = [(14, 2.2, 0.9), (38, 3.0, 1.0), (62, 2.2, 0.8), (80, 3.0, 1.0), (92, 2.2, 0.7)]
    for i, (dy, sw, frac) in enumerate(rows):
        y = horizon + dy * s
        spans = [(8 * s, W - 8 * s)] if y < horizon + 30 * s else [(8 * s, box[0]), (box[1], W - 8 * s)]
        for k, (x0, x1) in enumerate(spans):
            # a long span becomes pieces with gaps, so no line runs the width
            pieces = []
            while x1 - x0 > 260 * s:
                pieces.append((x0, x0 + 170 * s))
                x0 += 200 * s
            pieces.append((x0, x1))
            for j, (a, b) in enumerate(pieces):
                dx = ((i * 11 + k * 17 + j * 7) % 29) * s
                a2, b2 = a + dx, a + dx + (b - a) * frac - dx
                if b2 - a2 > 8 * s:
                    els.append(El("stroke", wave(a2, b2, y, 2.2 * s, 34 * s, i + k + j), P["line"], 0.95, width=sw * s))
    if wordmark:
        # (size, right margin, baseline): the name set flush right
        size, margin, baseline = wordmark
        wm, adv = letters(NEXIUM, size, 0, 0)
        wm, adv = letters(NEXIUM, size, W - adv - margin, baseline)
        els.append(El("fill", wm, P["wordmark"]))
    return P["paper"], els


# ---- SVG -------------------------------------------------------------------------

def svg_body(els, prefix):
    defs, body = [], []
    clips = {}
    for el in els:
        attrs = ""
        if el.clip is not None:
            key = write(el.clip)
            if key not in clips:
                clips[key] = f"{prefix}c{len(clips)}"
                defs.append(f'<clipPath id="{clips[key]}"><path d="{key}"/></clipPath>')
            attrs += f' clip-path="url(#{clips[key]})"'
        if el.opacity < 1:
            attrs += f' opacity="{el.opacity}"'
        if el.kind == "fill":
            body.append(f'<path d="{write(el.cmds)}" fill="{el.color}"{attrs}/>')
        else:
            body.append(f'<path d="{write(el.cmds)}" fill="none" stroke="{el.color}" stroke-width="{el.width:.2f}" '
                        f'stroke-linecap="round" stroke-linejoin="round"{attrs}/>')
    return "".join(defs), "".join(body)


def svg(W, H, scenes, rx=0, title="Nexium"):
    """One SVG. scenes: [(class, background, elements)]; with two, the first
    shows by day and the second under prefers-color-scheme: dark."""
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-label="{title}">']
    if len(scenes) > 1:
        out.append("<style>.night{display:none}@media (prefers-color-scheme:dark){.day{display:none}.night{display:inline}}</style>")
    out.append("<defs>")
    if rx:
        out.append(f'<clipPath id="corner"><path d="{write(rect(0, 0, W, H, rx))}"/></clipPath>')
    bodies = []
    for i, (cls, bg, els) in enumerate(scenes):
        defs, body = svg_body(els, f"{cls[0]}")
        out.append(defs)
        bodies.append((cls, bg, body))
    out.append("</defs>")
    for cls, bg, body in bodies:
        g = f' class="{cls}"' if len(scenes) > 1 else ""
        clip = ' clip-path="url(#corner)"' if rx else ""
        out.append(f'<g{g}{clip}><rect width="{W}" height="{H}" fill="{bg}"/>{body}</g>')
    out.append("</svg>\n")
    return "".join(out)


# ---- raster: the scene drawn with Pillow, supersampled ------------------------

def hexrgb(c):
    return tuple(int(c[i:i + 2], 16) for i in (1, 3, 5))


def mask_of(el, size, S):
    """The element's coverage as an L image at S times the size."""
    m = Image.new("L", size, 0)
    polys = flatten(mapped(el.cmds, lambda x, y: (x * S, y * S)), steps=16)
    if el.kind == "fill":
        for pts, _ in polys:
            if len(pts) < 3:
                continue
            one = Image.new("L", size, 0)
            ImageDraw.Draw(one).polygon(pts, fill=255)
            m = ImageChops.difference(m, one)  # even-odd, as the letters need
    else:
        d = ImageDraw.Draw(m)
        w = max(1, round(el.width * S))
        for pts, closed in polys:
            if closed:
                pts = pts + [pts[0]]
            d.line(pts, fill=255, width=w, joint="curve")
            for (x, y) in (pts[0], pts[-1]):
                d.ellipse((x - w / 2, y - w / 2, x + w / 2, y + w / 2), fill=255)
    if el.clip is not None:
        c = Image.new("L", size, 0)
        for pts, _ in flatten(mapped(el.clip, lambda x, y: (x * S, y * S)), steps=16):
            if len(pts) >= 3:
                ImageDraw.Draw(c).polygon(pts, fill=255)
        m = ImageChops.multiply(m, c)
    if el.opacity < 1:
        m = m.point(lambda v, o=el.opacity: int(v * o))
    return m


def raster(W, H, bg, els, S=4, rx=0, circle=False):
    size = (W * S, H * S)
    canvas = Image.new("RGB", size, hexrgb(bg))
    for el in els:
        canvas.paste(hexrgb(el.color), mask=mask_of(el, size, S))
    out = canvas.resize((W, H), Image.LANCZOS)
    if rx or circle:
        a = Image.new("L", size, 0)
        if circle:
            ImageDraw.Draw(a).ellipse((0, 0, size[0] - 1, size[1] - 1), fill=255)
        else:
            ImageDraw.Draw(a).rounded_rectangle((0, 0, size[0] - 1, size[1] - 1), radius=rx * S, fill=255)
        out = out.convert("RGBA")
        out.putalpha(a.resize((W, H), Image.LANCZOS))
    return out


# ---- the files ------------------------------------------------------------------

def square(P, size):
    s = size / 256
    return scene(P, size, size, 160 * s, 128 * s, 264 * s, s)


def banner(P):
    bg, els = scene(P, 880, 260, 160, 236, 568, 1.0, sun=(412, 112), wordmark=(84, 40, 118))
    return bg, els


def social(P):
    """The social preview: the scene on the left, and on the right the
    name, what Nexium is in a sentence, what one source ships as, the site."""
    bg, els = scene(P, 1280, 640, 440, 320, 700, 2.1, sun=(548, 262))
    x = 712
    wm, adv = letters(NEXIUM, 128, x, 214)
    els.append(El("fill", wm, P["wordmark"]))
    lines = [("Native code through C, with effects", 33, 270), ("the compiler checks.", 33, 312),
             ("Ships as a C library, a Python wheel, a Rust", 27, 362), ("crate, an npm package or WebAssembly.", 27, 396)]
    for text, size, baseline in lines:
        cmds, _ = prose(text, size, x, baseline)
        els.append(El("fill", cmds, P["wordmark"]))
    url, w = prose("londopy.github.io/nexium", 27, x, 0)
    url, w = prose("londopy.github.io/nexium", 27, 1280 - 60 - w, 600)
    els.append(El("fill", url, P["wordmark"]))
    return bg, els


def small_scene(P, size):
    """The mark at sizes where letters fail: the peak with its cap, the
    sky, the water with the mirrored flank, the sun as a dot."""
    s = size / 256
    horizon, top, cx = 176 * s, 46 * s, 128 * s
    els = [El("fill", rect(0, 0, size, 26 * s), P["skywash"]),
           El("fill", circle(198 * s, 92 * s, 17 * s), P["disc"], P["disc_op"])]
    peak = fuji(-30 * s, 286 * s, cx, horizon, top)
    els.append(El("fill", peak, P["mountain"]))
    els.append(El("fill", cap(cx, top, horizon - top), P["snow"]))
    els.append(El("fill", rect(0, horizon, size, size - horizon), P["water"]))
    inverted = mapped(peak, lambda x, y: (x, horizon + (horizon - y) * 0.55))
    els.append(El("fill", inverted, P["reflection"], 0.45))
    return P["paper"], els


def circle_icon(P, size):
    """The scene composed for a circle: the horizon higher and the letters
    closer under the mountain, so the rim cuts only sky and water."""
    s = size / 256 * 0.74
    return scene(P, size, size, size * 0.60, size / 2, size * 1.02, s)


def wizard_large(P):
    # Inno Setup's tall image beside the pages: the mark, the name under it
    s = 0.62
    bg, els = scene(P, 164, 314, 150, 82, 172, s)
    wm, adv = letters(NEXIUM, 30, 0, 0)
    wm, adv = letters(NEXIUM, 30, (164 - adv) / 2, 262)
    els.append(El("fill", wm, P["wordmark"]))
    return bg, els


def wizard_small(P):
    s = 0.21
    return scene(P, 55, 58, 34, 27, 60, s)


def save_text(rel, text):
    path = os.path.join(ROOT, rel)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("wrote", rel)


def main(groups):
    want = lambda g: not groups or g in groups
    if want("logo"):
        d_bg, d_els = square(DAY, 256)
        n_bg, n_els = square(NIGHT, 256)
        save_text("assets/logo.svg", svg(256, 256, [("day", d_bg, d_els), ("night", n_bg, n_els)], rx=28, title="Nexium: Fuji over a lake, reflected as nx"))
    if want("banner"):
        for name, P in (("light", DAY), ("dark", NIGHT)):
            bg, els = banner(P)
            save_text(f"assets/banner-{name}.svg", svg(880, 260, [("one", bg, els)], rx=16))
    if want("social"):
        bg, els = social(DAY)
        save_text("assets/social-preview.svg", svg(1280, 640, [("one", bg, els)]))
        raster(1280, 640, bg, els, S=3).save(os.path.join(ROOT, "assets", "social-preview.png"), optimize=True)
        print("wrote assets/social-preview.png")
    if want("icon"):
        sizes = [16, 24, 32, 48, 64, 128, 256]
        images = {}
        for n in sizes:
            # the title bar and the taskbar get the small composition
            bg, els = small_scene(DAY, n) if n <= 32 else square(DAY, n)
            images[n] = raster(n, n, bg, els, S=8 if n <= 64 else 4, rx=n * 0.11)
        images[256].save(os.path.join(ROOT, "assets", "icon-256.png"), optimize=True)
        images[256].save(os.path.join(ROOT, "editors", "vscode", "icon.png"), optimize=True)
        images[256].save(os.path.join(ROOT, "assets", "icon.ico"), sizes=[(n, n) for n in sizes],
                         append_images=[images[n] for n in sizes if n != 256])
        print("wrote assets/icon-256.png, assets/icon.ico, editors/vscode/icon.png")
    if want("circle"):
        bg, els = circle_icon(DAY, 1024)
        raster(1024, 1024, bg, els, S=3, circle=True).save(os.path.join(ROOT, "assets", "icon-circle-1024.png"), optimize=True)
        print("wrote assets/icon-circle-1024.png")
    if want("small"):
        d_bg, d_els = small_scene(DAY, 256)
        n_bg, n_els = small_scene(NIGHT, 256)
        save_text("assets/icon-small.svg", svg(256, 256, [("day", d_bg, d_els), ("night", n_bg, n_els)], rx=28, title="Nexium"))
        sizes = [16, 24, 32, 48, 64]
        images = {}
        for n in sizes:
            bg, els = small_scene(DAY, n)
            images[n] = raster(n, n, bg, els, S=8, rx=n * 0.11)
        images[64].save(os.path.join(ROOT, "assets", "icon-small-64.png"), optimize=True)
        images[64].save(os.path.join(ROOT, "assets", "icon-small.ico"), sizes=[(n, n) for n in sizes],
                        append_images=[images[n] for n in sizes if n != 64])
        print("wrote assets/icon-small.svg, assets/icon-small.ico, assets/icon-small-64.png")
    if want("installer"):
        bg, els = wizard_large(DAY)
        raster(164, 314, bg, els, S=4).convert("RGB").save(os.path.join(ROOT, "installers", "windows", "wizard-large.bmp"))
        bg, els = wizard_small(DAY)
        raster(55, 58, bg, els, S=8).convert("RGB").save(os.path.join(ROOT, "installers", "windows", "wizard-small.bmp"))
        print("wrote installers/windows/wizard-large.bmp, wizard-small.bmp")


if __name__ == "__main__":
    main(sys.argv[1:])
