Please donot use this for commercial purposes
This is a study project to search through new indian law document
https://www.mha.gov.in/sites/default/files/250883_english_01042024.pdf

Before
Make sure you have docker and rust in your system.
`docker compose up qdrant`

Step #1 (This part can be automated)
Feed the law document to database
> note
> This PDF is unstrcutured and quite hard to parse so the following step is idiosyncratic
> Prepare data
> * Download the PDF
> * Convert it into text -> https://www.xpdfreader.com/pdftotext-man.html -> `laws.txt`
> * Move text file to `./data`

Split it into sections
You need to do some manual work, because data in PDF is unstrctured
Or maybe you can use one of the paid LLMs to parse the PDF into structured data

Step #2
Keep structured data in text_files `./data/formatted`
  * See format.example to see the example
  * Ideally seperate by chapter, but it doesnt matter
Convert structured data to embedding using sentence transformer(see law_search repo)
Save embedding into vector DB(see law_search repo)

Step #3
Use webserver to query from vecterDb(see web_server)
  * Webserver is rocket

---

We use Qdrant as vectorDB
Postgres to save structured data
~Candle-rs + GritLM/GritLM-8x7B (if possible) for sentence transformer~
Used fastembed + BGELargeENV15
