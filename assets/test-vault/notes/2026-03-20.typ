#import "../vault.typ": *

#show: journal.with(
  title: "2026-03-20",
  date: "2026-03-20",
  previous: "@2026-03-19",
)

#let current-pet-project = xlink-scope.with(
  also: "work/pet project", 
  props: ("status",)
)

== Start working
#align(center)[
  Working on two tasks in the same time
]

#grid(
  columns: (1fr, 1fr),
  align(center)[
    #current-work[
      14:30 start doing task \
      #xlink("some new feature")
      
    ]
  ],
  align(center)[
    #current-pet-project[
      14:45 started claude code session \
      for planning pet project task \
      #xlink("add-tauri-app")
    ]
  ]
)

// #grid(
//   columns: (1fr, 1fr),
//   align(center)[
//     some text
//   ],
//   align(right)[
//     text aligned right with #xlink("programming") \
//     adding #current-work[
//       some text and #xlink("build-mvp")
//     ]
//   ]
// )
