import sqlite3
from sqlite3 import Error


def create_connection():
    """ create a database connection to the SQLite database
    specified by db_file
    :param db_file: database file
    :return: Connection object or None
    """
    database = r"../streamer/signatures_db.db"

    conn = None
    try:
        conn = sqlite3.connect(database, uri=True)
        return conn
    except Error as e:
        print(e)

    return conn

	
def get_last_id(conn):
    cur = conn.cursor()
    try:
        cur.execute("SELECT max(id) from signatures")
    except Error as e:
        print(e)
        return None
		
    r = cur.fetchone()
    cur.close()
    if (r[0] == None):
        return None
		
    return r[0]

def get_db_row(conn, row_id):
    cur = conn.cursor()
    try:
        cur.execute("SELECT * FROM signatures WHERE id=?", (row_id,))
    except Error as e:
        print(e)
        return None
		
    r = cur.fetchone()
    cur.close()
    return r

	
# the BETWEEN operation is inclusive of start and end
def get_rows_for_block_range(conn, start_block, end_block):
    cur = conn.cursor()
    cur.execute("SELECT * FROM signatures WHERE block_slot BETWEEN ? AND ?", (start_block, end_block))
    r = cur.fetchall()
    cur.close()
    return r
	
	
def get_first_block_id(conn):
	first_row = get_db_row(conn, 0)
	if(first_row == None):
		return -1
		
	return first_row[1]
	
def get_last_block_id(conn):
	last_id = get_last_id(conn)
	
	if(last_id == None):
		return -1
		
	last_row = get_db_row(conn, last_id)
	return last_row[1]