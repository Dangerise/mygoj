import os
import sqlite3

os.makedirs("tmp", exist_ok=True)
os.remove("tmp/empty.db")

con = sqlite3.connect("./tmp/empty.db")

with open("crates/server/src/sql/create.sql") as f:
    sql = f.read()
    con.executescript(sql)

con.commit()
con.close()
