import sqlite3

con = sqlite3.connect("/home/dangerise/mygoj/data.db")

with open("crates/server/src/sql/create.sql") as f:
    sql = f.read()
    con.executescript(sql)
con.commit()
con.close()
