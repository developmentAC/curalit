// Some definitions presupposed by pandoc's typst output.
#let blockquote(body) = [
  #set text( size: 0.92em )
  #block(inset: (left: 1.5em, top: 0.2em, bottom: 0.2em))[#body]
]

#let horizontalrule = line(start: (25%,0%), end: (75%,0%))

#let endnote(num, contents) = [
  #stack(dir: ltr, spacing: 3pt, super[#num], contents)
]

#show terms: it => {
  it.children
    .map(child => [
      #strong[#child.term]
      #block(inset: (left: 1.5em, top: -0.4em))[#child.description]
      ])
    .join()
}

// Some quarto-specific definitions.

#show raw.where(block: true): set block(
    fill: luma(230),
    width: 100%,
    inset: 8pt,
    radius: 2pt
  )

#let block_with_new_content(old_block, new_content) = {
  let d = (:)
  let fields = old_block.fields()
  fields.remove("body")
  if fields.at("below", default: none) != none {
    // TODO: this is a hack because below is a "synthesized element"
    // according to the experts in the typst discord...
    fields.below = fields.below.abs
  }
  return block.with(..fields)(new_content)
}

#let empty(v) = {
  if type(v) == str {
    // two dollar signs here because we're technically inside
    // a Pandoc template :grimace:
    v.matches(regex("^\\s*$")).at(0, default: none) != none
  } else if type(v) == content {
    if v.at("text", default: none) != none {
      return empty(v.text)
    }
    for child in v.at("children", default: ()) {
      if not empty(child) {
        return false
      }
    }
    return true
  }

}

// Subfloats
// This is a technique that we adapted from https://github.com/tingerrr/subpar/
#let quartosubfloatcounter = counter("quartosubfloatcounter")

#let quarto_super(
  kind: str,
  caption: none,
  label: none,
  supplement: str,
  position: none,
  subrefnumbering: "1a",
  subcapnumbering: "(a)",
  body,
) = {
  context {
    let figcounter = counter(figure.where(kind: kind))
    let n-super = figcounter.get().first() + 1
    set figure.caption(position: position)
    [#figure(
      kind: kind,
      supplement: supplement,
      caption: caption,
      {
        show figure.where(kind: kind): set figure(numbering: _ => numbering(subrefnumbering, n-super, quartosubfloatcounter.get().first() + 1))
        show figure.where(kind: kind): set figure.caption(position: position)

        show figure: it => {
          let num = numbering(subcapnumbering, n-super, quartosubfloatcounter.get().first() + 1)
          show figure.caption: it => {
            num.slice(2) // I don't understand why the numbering contains output that it really shouldn't, but this fixes it shrug?
            [ ]
            it.body
          }

          quartosubfloatcounter.step()
          it
          counter(figure.where(kind: it.kind)).update(n => n - 1)
        }

        quartosubfloatcounter.update(0)
        body
      }
    )#label]
  }
}

// callout rendering
// this is a figure show rule because callouts are crossreferenceable
#show figure: it => {
  if type(it.kind) != str {
    return it
  }
  let kind_match = it.kind.matches(regex("^quarto-callout-(.*)")).at(0, default: none)
  if kind_match == none {
    return it
  }
  let kind = kind_match.captures.at(0, default: "other")
  kind = upper(kind.first()) + kind.slice(1)
  // now we pull apart the callout and reassemble it with the crossref name and counter

  // when we cleanup pandoc's emitted code to avoid spaces this will have to change
  let old_callout = it.body.children.at(1).body.children.at(1)
  let old_title_block = old_callout.body.children.at(0)
  let old_title = old_title_block.body.body.children.at(2)

  // TODO use custom separator if available
  let new_title = if empty(old_title) {
    [#kind #it.counter.display()]
  } else {
    [#kind #it.counter.display(): #old_title]
  }

  let new_title_block = block_with_new_content(
    old_title_block, 
    block_with_new_content(
      old_title_block.body, 
      old_title_block.body.body.children.at(0) +
      old_title_block.body.body.children.at(1) +
      new_title))

  block_with_new_content(old_callout,
    block(below: 0pt, new_title_block) +
    old_callout.body.children.at(1))
}

// 2023-10-09: #fa-icon("fa-info") is not working, so we'll eval "#fa-info()" instead
#let callout(body: [], title: "Callout", background_color: rgb("#dddddd"), icon: none, icon_color: black, body_background_color: white) = {
  block(
    breakable: false, 
    fill: background_color, 
    stroke: (paint: icon_color, thickness: 0.5pt, cap: "round"), 
    width: 100%, 
    radius: 2pt,
    block(
      inset: 1pt,
      width: 100%, 
      below: 0pt, 
      block(
        fill: background_color, 
        width: 100%, 
        inset: 8pt)[#text(icon_color, weight: 900)[#icon] #title]) +
      if(body != []){
        block(
          inset: 1pt, 
          width: 100%, 
          block(fill: body_background_color, width: 100%, inset: 8pt, body))
      }
    )
}



#let article(
  title: none,
  subtitle: none,
  authors: none,
  date: none,
  abstract: none,
  abstract-title: none,
  cols: 1,
  margin: (x: 1.25in, y: 1.25in),
  paper: "us-letter",
  lang: "en",
  region: "US",
  font: "libertinus serif",
  fontsize: 11pt,
  title-size: 1.5em,
  subtitle-size: 1.25em,
  heading-family: "libertinus serif",
  heading-weight: "bold",
  heading-style: "normal",
  heading-color: black,
  heading-line-height: 0.65em,
  sectionnumbering: none,
  pagenumbering: "1",
  toc: false,
  toc_title: none,
  toc_depth: none,
  toc_indent: 1.5em,
  doc,
) = {
  set page(
    paper: paper,
    margin: margin,
    numbering: pagenumbering,
  )
  set par(justify: true)
  set text(lang: lang,
           region: region,
           font: font,
           size: fontsize)
  set heading(numbering: sectionnumbering)
  if title != none {
    align(center)[#block(inset: 2em)[
      #set par(leading: heading-line-height)
      #if (heading-family != none or heading-weight != "bold" or heading-style != "normal"
           or heading-color != black or heading-decoration == "underline"
           or heading-background-color != none) {
        set text(font: heading-family, weight: heading-weight, style: heading-style, fill: heading-color)
        text(size: title-size)[#title]
        if subtitle != none {
          parbreak()
          text(size: subtitle-size)[#subtitle]
        }
      } else {
        text(weight: "bold", size: title-size)[#title]
        if subtitle != none {
          parbreak()
          text(weight: "bold", size: subtitle-size)[#subtitle]
        }
      }
    ]]
  }

  if authors != none {
    let count = authors.len()
    let ncols = calc.min(count, 3)
    grid(
      columns: (1fr,) * ncols,
      row-gutter: 1.5em,
      ..authors.map(author =>
          align(center)[
            #author.name \
            #author.affiliation \
            #author.email
          ]
      )
    )
  }

  if date != none {
    align(center)[#block(inset: 1em)[
      #date
    ]]
  }

  if abstract != none {
    block(inset: 2em)[
    #text(weight: "semibold")[#abstract-title] #h(1em) #abstract
    ]
  }

  if toc {
    let title = if toc_title == none {
      auto
    } else {
      toc_title
    }
    block(above: 0em, below: 2em)[
    #outline(
      title: toc_title,
      depth: toc_depth,
      indent: toc_indent
    );
    ]
  }

  if cols == 1 {
    doc
  } else {
    columns(cols, doc)
  }
}

#set table(
  inset: 6pt,
  stroke: none
)

#show: doc => article(
  pagenumbering: "1",
  toc_title: [Table of contents],
  toc_depth: 3,
  cols: 1,
  doc,
)

// Print canvas: 46.78 in wide by 33.06 in high (landscape).
// Adjust the page margins here if a printing vendor requests a different safe area.
#set page(width: 46.78in, height: 33.06in, margin: (x: 0.52in, y: 0.12in), fill: rgb("#f8fafb"))
// Arial is used for readable, familiar large-format typography.
#set text(font: "Arial", size: 18pt, fill: rgb("#142b4a"))
#set par(leading: 0.34em)

// Poster palette: update these named colors to retheme section accents consistently.
#let navy = rgb("#063574")
#let cyan = rgb("#12b9d5")
#let lime = rgb("#75df34")
#let yellow = rgb("#f3df3f")
#let pink = rgb("#ec5ab5")
#let ink = rgb("#142b4a")
#let muted = rgb("#e8f1f4")
// Banner palette: adjust the two stops and angle to change the sky-blue-to-yellow transition.
#let banner-sky = rgb("#75cbed")
#let banner-yellow = rgb("#f6de54")
#let banner-gradient = gradient.linear(banner-sky, banner-yellow, angle: 0deg)

#let section(title, accent, body) = box(
  width: 100%,
  inset: 13pt,
  radius: 7pt,
  fill: white,
  stroke: (paint: rgb("#b9cad5"), thickness: 1.2pt),
)[
  #block(width: 100%, inset: (x: 12pt, y: 7pt), fill: accent, radius: 4pt)[
    #text(size: 25pt, weight: "bold", fill: navy)[#title]
  ]
  #v(6pt)
  #body
]

#let figure(path, caption, width: 94%) = [
  #align(center)[#image(path, width: width)]
  #v(4pt)
  #align(center)[#text(size: 18pt, fill: rgb("#38516b"), style: "italic")[#caption]]
]

#let term(name, definition) = [
  #text(weight: "bold", fill: navy)[#name:] #definition
]
// heigh was 4.1in
// Header controls: alter `height`, `dx`, `dy`, and image widths to reposition the logo or QR code.
#block(width: 100%, height: 6.1in, inset: 0pt, fill: banner-gradient, radius: 8pt)[
  #place(top + left, dx: 30pt, dy: 28pt)[
    #box(fill: white, inset: 10pt, radius: 4pt, stroke: (paint: navy, thickness: 1.5pt))[
      #image("assets/logo_right.png", width: 7.1in) //width was 5.1in
    ]
  ]
  #align(center + horizon)[
  // Title position: lower this `v()` value to move the title farther upward.
//  #v(0.18in)
  #v(1.0in)
  #text(size: 86pt, weight: "bold", fill: navy)[CuraLit]
//  #v(2pt)
  #v(.03pt) //text size was 35pt
  #text(size: 45pt, weight: "bold", fill: navy)[An Open-Source Hybrid System Integrating PubMed Filtering, RAG, and LLM Models for Visualized Research Guidance]
//  #v(12pt)
  #v(0.1pt)
  #text(size:30pt, fill: navy)[Oliver Bonham-Carter, PhD; Allegheny College, Meadville, PA]
  #v(0.1pt)
  #text(size:30pt, fill: navy)[Vincent Mametjanov; Meadville Area High School, Meadville, PA]

  ]
  // QR code is isolated in a white panel for reliable scanning on the colored banner.
  #place(top + right, dx: -34pt, dy: 36pt)[
    #align(right)[
      #box(fill: white, inset: 10pt, radius: 4pt, stroke: (paint: navy, thickness: 1.5pt))[
        #image("assets/curalit_QR.png", width: 4.2in) //width was 2.2in
      ]
//      #v(0.2pt) //was 5pt
//      #text(size: 14pt, fill: navy)[Great Plains Bioinformatics Conference]
//      #text(size: 13pt, fill: navy)[September 28-30, 2026 | Omaha, Nebraska]
    ]
  ]
]

// END OF BANNER


//#v(0.16in)
     #v(0.5in) //obc add to lower the flowchart and make it more centered in the column


// TOP OF SECOND COLUMN

#grid(
  columns: (1fr, 1fr, 1fr),
  gutter: 0.20in,
  [
    #section("Introduction", cyan, [
      #text(size: 22pt, weight: "bold", fill: navy)[Help early-career researchers move from a broad curiosity to an evidence-grounded research project using AI technology.]
      #v(9pt)
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Undergraduates and novice investigators often have trouble developing research projects, despite having areas of interest]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Researchers are overwhelmed by the volume of information returned by online search engines]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[The relationships between articles and their interests are not always clear]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Researchers often need to discuss (e.g., brainstorm) ideas before they can develop a project with confidence]
//      #list(marker: [#text(fill: cyan)[●]], spacing: 8pt)[TODO]
//      #list(marker: [#text(fill: cyan)[●]], spacing: 8pt)[TODO]
//      #list(marker: [#text(fill: cyan)[●]], spacing: 8pt)[TODO]


      #v(8pt)
      #term("CuraLit", "a local, open-source workflow that turns a keyword-filtered PubMed collection into an explorable research knowledge base.")

      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Keywords related to an area of interest are parsed against the PubMed (https://pubmed.ncbi.nlm.nih.gov/) corpus to identify a subset of articles]
//      #list(marker: [#text(fill: cyan)[●]], spacing: 8pt)[Subset is then used to create LLMs]
//      #list(marker: [#text(fill: cyan)[●]], spacing: 8pt)[Build a searchable SQLite database from PubMed XML files for fact verification]
//ß      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Filter large PubMed datasets by keyword]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Analyze the effectiveness of keywords in retrieving articles from the PubMed corpus]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Visualize research trends with interactive plots]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[Generate custom LLMs with Ollama or LM Studio for:

      #list(marker: [#text(fill: lime)[●]], spacing: 6pt)[Answering questions about specific research domains]
      #list(marker: [#text(fill: lime)[●]], spacing: 6pt)[Brainstorming ideas and generating new insights]
//      #list(marker: [#text(fill: lime)[●]], spacing: 6pt)[Creating foundational language accessible to novices and experts]
      #list(marker: [#text(fill: lime)[●]], spacing: 6pt)[Choosing research topics and chatting about its design]
      #list(marker: [#text(fill: lime)[●]], spacing: 6pt)[Developing hypotheses]
      #list(marker: [#text(fill: lime)[●]], spacing: 6pt)[Synthesizing literature reviews]
      ]
    ])


      #v(0.10in)

      #section("Features", lime, [
      #text(size: 19pt, weight: "bold", fill: navy)[A resource to help early-career researchers move from a broad curiosity to an evidence-grounded project]
      #v(6pt)

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[Memory-Efficient Processing
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Written in Rust to process millions of articles without performance degradation]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[Flexible Keyword Matching
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Search across PubMed (https://pubmed.ncbi.nlm.nih.gov/): titles, abstracts, MeSH terms, chemicals, and authors]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[Checkpoint System
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Continue searches from where you left off]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[Statistical Analysis
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Recommendations for keyword refinement]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[Interactive Visualizations
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Interactive HTML plots: heatmaps, scatter plots, and histograms to help researchers focus on specific searches]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[LLM Generation
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Generate Ollama model files for research conversations (https://www.ollama.com)]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[RAG (Retrieval-Augmented Generation)
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[File-based Qdrant storage using Docker (http://www.docker.com); no separate server is required]
      ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[SQLite Database for Fact Verification
        #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Database containing article details and citations to support fact-checking and reduce hallucinations]
      ]
])

 
     #v(0.03in)
    #section("Traditional Searches Can Frustrate and Reduce Productivity", pink, [
           #v(0.24in) //obc add to lower the flowchart and make it more centered in the column

      #grid(
        columns: (1.1fr, 1.1fr, 0fr),
        gutter: 7pt,
        [#figure("assets/stress.png", "",width: 100%)],
        [#figure("assets/scholar_results.png", "", width: 100%)],

      )
      #v(0.1pt)
      #align(center)[#text(size: 18pt, fill: rgb("#38516b"), style: "italic")[Indicated on the left, many students believe that success in research depends only on their skills to do the work of the project. They may not realize that the successes of their projects are initially based on the foundational work they do during development. CuraLit reduces the frustration of interpreting search-engine results by applying visual and AI-assisted guidance to help students navigate the literature and its concepts. Shown right is a typical keyword search with nearly 800,000 results which may lead students to feel overwhelmed.]]
    ])

  ],
// END OF FIRST COLUMN


// TOP OF SECOND COLUMN
  [

     #v(0.0in) //obc add to lower the flowchart and make it more centered in the column
    #section("Flowchart", orange, [
     #v(1.0in) //obc add to lower the flowchart and make it more centered in the column

      #grid(
        columns: (1.1fr, 0fr, 0fr),
        gutter: 7pt,
        [#figure("assets/flowchart.png", "", width: 94%)],
      )
      #v(3pt)
      #align(center)[#text(size: 18pt, fill: rgb("#38516b"), style: "italic")[Project workflow: CuraLit filters a PubMed corpus according to user-selected keywords. It then builds a database from article details, methods, citations, and authors, and uses that knowledge to create model files for AI tools. Investigators can chat with the model through Ollama to explore concepts and develop an original research project. Statistics and visualizations provide feedback on keyword effectiveness, while the local database supports fact-checking.]]
    ])
     #v(0.5in) //obc add to lower the flowchart and make it more centered in the column

    #section("CuraLit Steps", yellow, [
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[1. Curate:] #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Choose terms that express a research interest; CuraLit filters PubMed XML files into a focused article collection.]
      ]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[2. Characterize:] #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[ Calculate article, journal, and MeSH-term summaries; generate editable visualizations.]
      ]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[3. Ground:] #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Build a searchable local knowledge base from the selected articles.]
        ]

      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[4. Question:] #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Retrieve relevant passages before an LLM drafts a response; display supporting PMIDs.]
      ]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[5. Verify:] #list(marker: [#text(fill: lime)[●]], spacing: 4pt)[Check cited PMIDs and article metadata against a local SQLite database.]
      ]
    ])


    //     #v(1.0in)
         #v(0.5in) //obc add to lower the flowchart and make it more centered in the column

    #section("Keyword Effectiveness in Retrieval", orange, [
     #v(0.5in) //obc add to lower the flowchart and make it more centered in the column

      #grid(
        columns: (1.1fr, 1.1fr, 0fr),
        gutter: 7pt,
        [#figure("assets/journals_piechart.png", "", width: 100%)],
        [#figure("assets/leading_mesh_terms.png", "", width: 100%)],
      )
      #v(3pt)
      #align(center)[#text(size: 18pt, fill: rgb("#38516b"), style: "italic")[These visualizations guide keyword selection. The pie chart (left) shows the leading journals among articles containing the selected keywords. The histogram (right) shows the most common MeSH terms. MeSH terms are standardized labels for biomedical concepts that support consistent searching. A separate author-frequency plot (not shown) also provides additional guidance.]]
    ])


  ],
// BOTTOM OF SECOND COLUMN



// TOP OF THIRD COLUMN
 [
    #section("Visual Guidance for Project Development", lime, [
      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[
        #text(weight: "bold")[MeSH frequency:] Identifies the clinical, population, and methodological concepts most represented in the corpus.]
      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[
        #text(weight: "bold")[Journal distribution:] Provides a quick view of publication venues relevant to the topic.]
      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[
        #text(weight: "bold")[Knowledge network:] Links terms and articles to expose clusters, connections, and potential avenues for reading.]
      #list(marker: [#text(fill: cyan)[●]], spacing: 4pt)[
        #text(weight: "bold")[Editable outputs:] Visualization scripts and data summaries can be adapted for a project, poster, or literature review.]
    ])


//         #v(1.0in)
     #v(1.0in) //obc add to lower the flowchart and make it more centered in the column

    #section("Launching the research process", orange, [
           #v(0.5in) //obc add to lower the flowchart and make it more centered in the column

      #grid(
        columns: (1.1fr, 1.1fr, 0fr),
        gutter: 7pt,
        [#figure("assets/1_program_start.png", "Step 1: Select the PubMed XML files to filter by keyword.", width: 100%)],//was width:96
        [#figure("assets/2_program_build_database.png", "Step 2: Build a database from the filtered articles.", width: 100%)],//was width:96



      )
      #grid(
        columns: (1.1fr, 1.1fr, 0fr),
        gutter: 7pt,
        [#figure("assets/3_program_RAG-generation.png", "Step 3: Generate a Retrieval-Augmented Generation (RAG) model from the filtered articles.", width: 100%)],//was width:96
        [#figure("assets/4_program_question_with_answer.png", "Step 4: Ask a question and receive an answer from the RAG model.", width: 94%)], //was width:84
      )

      #grid(
//        columns: (1.1fr, 1.1fr, 0fr),
        columns: (1.1fr, 1.1fr),
        gutter: 7pt,
        [#figure("assets/knowledge_network.png", "Step 5: Explore connections among filtered articles in a knowledge network.", width: 98%)],//was width:88
    

        [#figure("assets/journals2.png", "Step 6: View the distribution of filtered articles across journals.", width: 82%)],//was width:72
      )

      #v(3pt)
      #align(center)[#text(size: 15pt, fill: rgb("#38516b"), style: "italic")[]]
    ])
         #v(0.35in) //obc add to lower the flowchart and make it more centered in the column

    #section("Future Work", lime, [
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[Testing:] Test the tool to evaluate its effectiveness and usability.]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[User experience:] Conduct user testing to evaluate ease of use and overall satisfaction.]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[Resource consumption:] Optimize the CuraLit code and algorithms to reduce computational requirements.]
      #list(marker: [#text(fill: cyan)[●]], spacing: 6pt)[
        #text(weight: "bold")[Novice researcher feedback:] Gather feedback from novice researchers to improve the tool's functionality and user interface.]
    ])
 ]
)




