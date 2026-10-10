#let markersize = 26pt

#set rect(stroke: none)
#let marker = rect(width: markersize, height: markersize,
  rect(fill: white,
    inset: 1pt,
    rect(fill: black,
      inset: 2pt,
      rect(fill: white,
        inset: 2.5pt,
        rect(fill: black)
      ))
  ),
)

#set page(
  background: pad(16pt, pad(top: 2pt, bottom: 1pt, rect(
    fill: tiling(size: (14pt, 14pt))[
      #place(
        dx: 6pt,
        dy: 2pt,
        rotate(45deg, square(
          size: 1pt,
          fill: black,
        )
      )
    )
    ],
    width: 100%,
    height: 100%,
  ))),
  foreground: pad(8pt, {
    place(top +left, marker)
    place(top +right, marker)
    place(bottom +left, marker)
    place(bottom +right, marker)
  }),
  footer: place(center+bottom, pad(17pt, rect(
    fill: rgb("#ddd"),
    radius: 4pt,
    "Textury"
  )))
)
