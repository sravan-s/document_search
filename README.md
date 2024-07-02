Please donot use this for commercial purposes
This is a study project to search through new indian law document
https://www.mha.gov.in/sites/default/files/250883_english_01042024.pdf

Step #1
Feed the law document to database
PDF to Structured Data (id, summary, illustration, side_node)
You need to do some manual work, because data in PDF is unstrctured

Step #2
Convert structured data to embedding using sentence transformer
Save embedding into vector DB

Step #3
Use webserver to query from vecterDb

---

We use Qdrant as vectorDB
Postgres to save structured data
Candle-rs + GritLM/GritLM-8x7B (if possible) for sentence transformer 

