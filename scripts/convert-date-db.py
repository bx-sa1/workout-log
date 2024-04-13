#!/usr/bin/python3
import sqlite3
import datetime

db = sqlite3.connect("workouts.db")
res = db.execute("select date from workouts")

dates = []

while (date := res.fetchone()) != None :
    date = date[0]
    print("Original: " + date)

    iso = (datetime.datetime.strptime(date, "%Y-%m-%d %H:%M:%S") \
        .replace(tzinfo=datetime.timezone.utc) \
        .strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3]) \
        + 'Z'
    print("New: " + iso + "\n")

    dates.append((date, iso))

db.executemany("update workouts set date = ? where workouts.date = ?", [(date[1], date[0]) for date in dates]);

db.commit()
