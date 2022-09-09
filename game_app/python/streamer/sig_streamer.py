from pkgutil import get_data
from solana.rpc.api import Client
import concurrent.futures as cf
import numpy as np
import time

from sql_funcs import *
from rpc_funcs import *
from log import *

db_conn = create_database_connection(r"signatures_test_db.db")
# check the connection is valid
if db_conn is None:
	log_error("cannot create the database connection.")
	exit()

# connect to solana node
quick_node_dev = "my_node"

dev_client = Client(quick_node_dev)

if (not dev_client.is_connected()):
    log_error("cannot connect to quicknode endpoint.")
    exit()


current_row_id_to_insert = None
current_block = None

last_db_row = get_last_db_row(db_conn)

if (last_db_row != None):
    log("getting current_block from DB: " + str(last_db_row))

    current_row_id_to_insert = last_db_row[0] + 1
    current_block = last_db_row[1]

else:
    log("getting current_block from client")
    current_row_id_to_insert = 0
    current_block = get_slot(dev_client)

log_blue("Starting with row: " + str(current_row_id_to_insert) + " Current block: " + str(current_block))

last_signature = None
while(True):

    block_list, signature_list = get_block_list_and_signatures(quick_node_dev, current_block, last_signature)

     # if the last block in the list was the current block, just wait and check again shortly
    if(block_list[-1] == current_block):
        time.sleep(0.25)
        continue

    # we are only interested in the blocks after current_block so remove that one from the list
    block_list = block_list[1:]

    # only process max 1000 blocks at a time
    if (len(block_list) > 1000):
        block_list = block_list[:1000]

    signatures, last_signature = process_signatures(signature_list, last_signature, block_list[0], block_list[-1])

    # initialize a dictionary to hold any transactions
    blocks = {}
    for block in block_list:
        blocks[block] = []


    if (len(signatures) > 0):
        transactions = get_transactions(quick_node_dev, signatures)
        for t in transactions:
            slot = t["slot"]
            if (slot < block_list[0] or slot > block_list[-1]):
                continue

            if slot not in block_list:
                log_error("unknown slot ! " + str(slot))
                continue

            blocks[slot].append(t)
        
    max_block_buffer = 10
    block_buffer = min(max_block_buffer, len(block_list))
    # sometimes it can take a few blocks before the signatures actual become available
    # if the last block_buffer blocks are all empty then just cut them from the list
    # to try and avoid issues with asynchronous behaviour
    for idx in range(block_buffer):
        block_idx = block_list[-1]
        if (len(blocks[block_idx]) != 0):
            break

        log("remove empty block " + str(block_idx))
        block_list = block_list[:-1]
        
    rows_to_insert = []
    for block_idx in block_list:
        b_idx, data = get_data_from_block(block_idx, blocks[block_idx])
        current_row_id_to_insert = create_rows_from_data(current_row_id_to_insert, b_idx, data, rows_to_insert)

    insert_rows(db_conn, rows_to_insert)


    if (len(block_list) > 0):
        current_block = block_list[-1]