-- maybe just get category from company house sic codes? yes but they can have multiple sics, so still should store like this?
CREATE TABLE 'category' (
  id INTEGER PRIMARY KEY,
  'name' TEXT NOT NULL
);
INSERT INTO 'category' VALUES
(NULL, "groceries"),
(NULL, "bills");

CREATE TABLE IF NOT EXISTS "business" (
	"id"	INTEGER,
	"name"	TEXT NOT NULL,
	"companies_house_number"	INTEGER,
	"category_id"	INTEGER,
	PRIMARY KEY("id"),
	FOREIGN KEY("category_id") REFERENCES "category"("id")
);
INSERT INTO 'business' VALUES
(NULL, "Aldi", 02321869, NULL),
(NULL, "AO", NULL, NULL),
(NULL, "Morrisons", 00358949, NULL);

CREATE TABLE 'payees' (
  id INTEGER PRIMARY KEY,
  'raw_payee_name' TEXT NOT NULL,
  
  business_id INTEGER,

  FOREIGN KEY(business_id) REFERENCES business(id)
);
INSERT INTO 'payees' VALUES
(NULL, "Admiral Insurance", NULL),
(NULL, "Aldi", NULL),
(NULL, "Amazon", NULL),
(NULL, "Amazon Prime", NULL),
(NULL, "Argos", NULL),
(NULL, "Asda", NULL),
(NULL, "B&Q", NULL),
(NULL, "BT", NULL),
(NULL, "Boots", NULL),
(NULL, "British Gas", NULL),
(NULL, "Bulb", NULL),
(NULL, "Domino's", NULL),
(NULL, "Five Guys", NULL),
(NULL, "Forest Wf", NULL),
(NULL, "Franco Manca", NULL),
(NULL, "GWR", NULL),
(NULL, "Gatwick Airport", NULL),
(NULL, "Google", NULL),
(NULL, "Google Pay", NULL),
(NULL, "Great Western Railway", NULL),
(NULL, "Greensmiths Food", NULL),
(NULL, "Guardian Security", NULL),
(NULL, "HM Passport Office", NULL),
(NULL, "Jerk Cafe", NULL),
(NULL, "John Lewis & Partners", NULL),
(NULL, "Lakeland", NULL),
(NULL, "Marks & Spencer", NULL),
(NULL, "Matalan", NULL),
(NULL, "Microsoft", NULL),
(NULL, "Morgan Stanley", NULL),
(NULL, "Morrisons", NULL),
(NULL, "Mountain Warehouse", NULL),
(NULL, "National Railcards", NULL),
(NULL, "National Trust", NULL),
(NULL, "Ocado", NULL),
(NULL, "Pets At Home", NULL),
(NULL, "Post Office", NULL),
(NULL, "Sainsbury's", NULL),
(NULL, "South Eastern Railway", NULL),
(NULL, "South West Water", NULL),
(NULL, "South eastern railway", NULL),
(NULL, "Southeastern Railway", NULL),
(NULL, "Southern Water", NULL),
(NULL, "Spar", NULL),
(NULL, "Spotify", NULL),
(NULL, "Spotify Ab", NULL),
(NULL, "Stagecoach", NULL),
(NULL, "Tesco", NULL),
(NULL, "Thameslink", NULL),
(NULL, "The Range", NULL),
(NULL, "Thompson & Morgan", NULL),
(NULL, "Tk Maxx", NULL),
(NULL, "Topshop", NULL),
(NULL, "Transport for London", NULL),
(NULL, "Virgin Media", NULL),
(NULL, "WH Smith", NULL),
(NULL, "WHSmith", NULL),
(NULL, "Waitrose", NULL),
(NULL, "Waitrose & Partners", NULL),
(NULL, "Wasabi", NULL),
(NULL, "Whole Foods Market", NULL),
(NULL, "Wilko", NULL),
(NULL, "eBay", NULL),
(NULL, "www.nisbets.com", NULL);

CREATE TABLE 'addressx' (
  id INTEGER PRIMARY KEY,
  'name' TEXT NOT NULL,

  business_id INTEGER NOT NULL,

  FOREIGN KEY(business_id) REFERENCES business(id)
);
INSERT INTO 'addressx' VALUES
(NULL, "23 Street Manchester", 1),
(NULL, "23 Street London", 1),
(NULL, "23 Street Birmingham", 1);


