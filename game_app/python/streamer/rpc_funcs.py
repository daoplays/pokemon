import time
from borsh_construct import Enum, CStruct, U64
import base58
import requests
from requests.structures import CaseInsensitiveDict
import json as json
import numpy as np
import concurrent.futures as cf
from datetime import datetime
from log import *

button_type = Enum(
    "A",
    "B",
    "Up",
    "Down",
    "Left",
    "Right",
    "Start",
    "Select",
    enum_name = "Button"
)

charity_type = Enum(
    "EvidenceAction",
    "GirlsWhoCode",
    "OneTreePlanted",
    "OutrightActionInt",
    "TheLifeYouCanSave",
    "UkraineERF",
    "WaterOrg",
    enum_name = "Charity"
)
    
message = Enum(
"CreateDataAccount" / CStruct("amount" / U64),
"PushButton" / CStruct("button" / button_type, "amount" / U64),
"PlaceBid"/ CStruct("amount_charity" / U64, "amount_dao" / U64, "charity" / charity_type),
"SelectWinners",
"SendTokens",
enum_name="DPPInstruction", 
)


sleep_time = 0.25

def check_json_result(id, json_result):

	if ("error" in json_result.keys()):
		error = json_result["error"]
		log_error(str(id) + " returned error: " + str(error))
		return False

	if ("result" in json_result.keys()):
		return True

	return False

# returns the current slot
def get_slot(dev_client):
    while True:
        try:
            slot = dev_client.get_slot()
        except:
            log_error("get_slot transaction request timed out")
            time.sleep(sleep_time)
            continue

        if (not check_json_result("get_slot", slot)):
            time.sleep(sleep_time)
            continue

        break
		
    return slot["result"]


# returns the list of finalized blocks after and including block_idx
def sub_get_transactions(dev_client_url, signatures, have_transactions, transactions):


    headers = CaseInsensitiveDict()
    headers["Content-Type"] = "application/json"

    data_vec = []
    for i in range(len(signatures)):

        if (have_transactions[i]):
            continue

        new_request = json.loads('{"jsonrpc": "2.0","id": 1, "method":"getTransaction", "params":["GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me", {"encoding": "json", "maxSupportedTransactionVersion":0, "commitment": "confirmed"}]}')

        new_request["id"] = i + 1
        new_request["params"][0] = signatures[i]
        data_vec.append(new_request)
   
    print("submit transactions post request")
    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps(data_vec), timeout=10)
        except:
            log_error("getTransaction request timed out")
            time.sleep(sleep_time)
            continue

        if (resp.status_code != 200):
            log_error("getTransaction request unsuccessful")
            time.sleep(sleep_time)
            continue


        break

    if (resp.status_code != 200):
        return have_transactions, transactions

    response_json = resp.json()

    for response in response_json:
        if ("id" not in response.keys()):
            log_error("No id in getTransaction response")
            continue

        if ("error" in response.keys()):
            log_error("error in getTransaction response: " + str(response["error"]))
            continue

        if ("result" not in response.keys()):
            log_error("No result in getTransaction response")
            continue

        if (response["result"] == None):
            log_error("result is None in getTransaction response")
            continue

        id = response["id"]
        transactions[signatures[id - 1]] = response["result"]
        have_transactions[id - 1] = True

    
    return have_transactions, transactions

def get_one_transaction_batch(dev_client_url, signatures):
	
	batch_transactions = {}
	have_transactions = np.array([False] * len(signatures))
	while (len(np.array(signatures)[have_transactions == False]) != 0):
		log_blue("requesting " + str(len(np.array(signatures)[have_transactions == False]))  + " transactions")
		have_transactions, batch_transactions = sub_get_transactions(dev_client_url, signatures, have_transactions, batch_transactions)
		print(have_transactions)
			
	return batch_transactions


# Returns identity and transaction information about a confirmed block in the ledger
def get_transactions(dev_client_url, signatures):

    n_sigs = len(signatures)
    batch_size = 100
    # only submit max 100 requests in one go.  At some point this will start to timeout if too many are sent
    n_batches = n_sigs//batch_size + 1
    transactions = []

    if (n_batches == 1):
        batch_results = get_one_transaction_batch(dev_client_url, signatures)
    
    else:
        log_blue("requesting " + str(n_batches) + " with total " + str(n_sigs) + " signatures")

        batch_lists = []
        for batch in range(n_batches):
            batch_start = batch * batch_size
            batch_end = min(n_sigs, batch_start + batch_size)
            batch_block_list = signatures[batch_start : batch_end]
            batch_lists.append(batch_block_list)
			
        batch_results = {}
        with cf.ThreadPoolExecutor(10) as executor:
            futures = [executor.submit(get_one_transaction_batch, dev_client_url, batch_lists[batch_id]) for batch_id in range(n_batches)]

            for future in cf.as_completed(futures):
                # get the result for the next completed task
                batch_transactions = future.result() # blocks
                for key in batch_transactions.keys():
                    batch_results[key] = batch_transactions[key]

        
    for key in batch_results.keys():
            transactions.append(batch_results[key])
		
    return transactions

def perform_request_response_checks(function_name, response_json):

    if ("id" not in response_json.keys()):
        log_error("No id in " + function_name + " response")
        return False

    if ("error" in response_json.keys()):
        log_error("error in " + function_name + " response " + str(response_json["error"]))
        return False

    if ("result" not in response_json.keys()):
        log_error("No result in " + function_name + " response")
        return False

    if (response_json["result"] == None):
        log_error("result is None in " + function_name + " response")
        return False

    return True

def get_request_header():

    headers = CaseInsensitiveDict()
    headers["Content-Type"] = "application/json"
    headers["x-session-hash"] = "blabla"

    return headers

# returns the json request for getSignaturesForAddress
def get_signatures_request(current_signature = None, id = 1):

    # program = GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me
    # system  key  = 11111111111111111111111111111111
    new_request = json.loads('{"jsonrpc": "2.0","id": 1, "method":"getSignaturesForAddress", "params":["HMVfAm6uuwnPnHRzaqfMhLNyrYHxaczKTbzeDcjBvuDo", {"commitment": "confirmed"}]}')
    new_request["id"] = id
    if (current_signature != None):
        new_request["params"][1]["until"] = current_signature

    return new_request, get_request_header()

def process_signatures(signature_list, current_signature, min_slot = 0, max_slot = np.inf):

    # we need to update current_signature to the latest one whose slot doesn't exceed max_slot
    if (len(signature_list) > 0):
        for r in signature_list:
            if r["slot"] > max_slot:
                continue

            current_signature = r["signature"]
            break

    signatures = []
    for r in signature_list:

        if(r["slot"] < min_slot or r["slot"] > max_slot):
            print("sig outside slots: ", min_slot, max_slot, datetime.utcnow(), r)
            continue

        print(datetime.utcnow(), r)
        signatures.append(r["signature"])
        
    return signatures, current_signature

# returns the list of finalized blocks after and including block_idx
def get_signatures(dev_client_url, min_slot = 0, max_slot = np.inf, current_signature=None):

    if (current_signature != None):
        log_blue("requesting from signature " + str(current_signature))

    new_request, headers = get_signatures_request(current_signature)

   
    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps([new_request]), timeout=10)
        except:
            log_error("getSignaturesForAddress request timed out")
            time.sleep(sleep_time)
            continue

        if (resp.status_code != 200):
            log_error("getSignaturesForAddress request unsuccessful, try again")
            time.sleep(sleep_time)
            continue

        response_json = resp.json()[0]

        if (not perform_request_response_checks("getSignaturesForAddress", response_json)):
            time.sleep(sleep_time)
            continue

        break

    #print(response_json)
    result = response_json["result"]

    log_blue("have " + str(len(result)) + " signatures")

    # we need to update current_signature to the latest one whose slot doesn't exceed max_slot
    if (len(result) > 0):
        for r in result:
            if r["slot"] > max_slot:
                continue

            current_signature = r["signature"]
            break

    signatures = []
    for r in result:

        if(r["slot"] < min_slot or r["slot"] > max_slot):
            print("sig outside slots: ", min_slot, max_slot, datetime.utcnow(), r)
            continue

        print(datetime.utcnow(), r)
        signatures.append(r["signature"])
        
    return signatures, current_signature

# returns the json request for getBlocks
def get_block_list_request(current_block, id = 1):


    new_request = json.loads('{"jsonrpc": "2.0","id": 1, "method":"getBlocks", "params":[0, {"commitment": "confirmed"}]}')
    new_request["params"][0] = current_block
    new_request["id"] = id

    return new_request, get_request_header()

# returns the list of finalized blocks after and including block_idx
def get_block_list(dev_client_url, current_block):

    log_blue("requesting from block " + str(current_block))

    new_request, headers = get_block_list_request(current_block)
  
    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps([new_request]), timeout=10)
        except:
            log_error("getBlocks request timed out")
            time.sleep(sleep_time)
            continue

        if (resp.status_code != 200):
            log_error("getBlocks request unsuccessful")
            time.sleep(sleep_time)
            continue

        response_json = resp.json()[0]

        if (not perform_request_response_checks("getBlocks", response_json)):
            time.sleep(sleep_time)
            continue

        if (len(response_json["result"]) == 0):
            log_error("result is length zero in getBlocks response")
            time.sleep(sleep_time)
            continue


        break

    log_blue("have " + str(len(response_json["result"])) + " blocks")
    return response_json["result"]

def get_block_list_and_signatures(dev_client_url, current_block, current_signature  = None):

    log_blue("requesting from block " + str(current_block))

    if (current_signature != None):
        log_blue("requesting from signature " + str(current_signature))

    getBlocks_request, headers = get_block_list_request(current_block, 1)
    getSigs_request, headers = get_signatures_request(current_signature, 2)

    request_vec = []
    request_vec.append(getBlocks_request)
    request_vec.append(getSigs_request)

    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps(request_vec), timeout=10)
            
        except:
            log_error("getBlocks/getSignatures request timed out")
            time.sleep(sleep_time)
            continue

        
        if (resp.status_code != 200):
            log_error("getBlocks/getSignatures request unsuccessful")
            time.sleep(sleep_time)
            continue

        response_json = resp.json()
        
        if (len(response_json) != 2):
            log_error("getBlocks/getSignatures has incorrect reply length")
            time.sleep(sleep_time)
            continue

        

        if (response_json[0]["id"] == 1):
            getBlocks_response_json = response_json[0]
            getSigs_response_json = response_json[1]
        else:
            getBlocks_response_json = response_json[1]
            getSigs_response_json = response_json[0]

        if (not perform_request_response_checks("getBlocks", getBlocks_response_json)):
            time.sleep(sleep_time)
            continue

        if (not perform_request_response_checks("getSignatures", getSigs_response_json)):
            time.sleep(sleep_time)
            continue

        if (len(getBlocks_response_json["result"]) == 0):
            log_error("result is length zero in getBlocks response")
            time.sleep(sleep_time)
            continue

        break

    log_blue("have " + str(len(getBlocks_response_json["result"])) + " blocks")
    print(getBlocks_response_json["result"])
    
    log_blue("have " + str(len(getSigs_response_json["result"])) + " signatures")

    return getBlocks_response_json["result"], getSigs_response_json["result"]


def get_blocks_batch(dev_client_url, block_list, have_block, blocks):
    headers = CaseInsensitiveDict()
    headers["Content-Type"] = "application/json"

    data_vec = []
    id_map = {}
    for i in range(len(block_list)):

        if (have_block[i]):
            continue

        new_request = json.loads('{"jsonrpc": "2.0","id": 0, "method":"getBlock", "params":[0, {"encoding": "json","transactionDetails":"full", "rewards": false, "maxSupportedTransactionVersion":0, "commitment": "confirmed"}]}')

        new_request["id"] = i + 1
        new_request["params"][0] = block_list[i]
        data_vec.append(new_request)
        id_map[i + 1] = block_list[i]

    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps(data_vec), timeout=10)
        except:
            print("getBlock batch request timed out")
            time.sleep(sleep_time)
            continue
			
        break
    

    if (resp.status_code != 200):
        return have_block, blocks

    resp_json = resp.json()

    for response in resp_json:
        if ("id" not in response.keys()):
            continue

        if ("error" in response.keys()):
            continue

        if ("result" not in response.keys()):
            continue

        if (response["result"] == None):
            print(response)
            continue

        id = response["id"]
        blocks[block_list[id - 1]] = response["result"]
        have_block[id - 1] = True

    return have_block, blocks
    
def get_one_block_batch(dev_client_url, batch_block_list):
	
	batch_blocks = {}
	have_block = np.array([False] * len(batch_block_list))
	while (len(np.array(batch_block_list)[have_block == False]) != 0):
		print("requesting", len(batch_block_list), "blocks:", batch_block_list)
		have_block, batch_blocks = get_blocks_batch(dev_client_url, batch_block_list, have_block, batch_blocks)
		print(have_block)
			
	return batch_blocks
	
# Returns identity and transaction information about a confirmed block in the ledger
def get_blocks(dev_client_url, block_list):

	n_blocks = len(block_list)
	batch_size = 100
	# only submit max 100 requests in one go.  At some point this will start to timeout if too many are sent
	n_batches = n_blocks//batch_size + 1
	blocks = {}
	
	if (n_batches == 1):
		blocks = get_one_block_batch(dev_client_url, block_list)
    
	else:
		print("requesting ", n_batches, " with total ", n_blocks, " blocks")
    	
		batch_lists = []
		for batch in range(n_batches):
			batch_start = batch * batch_size
			batch_end = min(n_blocks, batch_start + batch_size)
			batch_block_list = block_list[batch_start : batch_end]
			batch_lists.append(batch_block_list)
			
		with cf.ThreadPoolExecutor(10) as executor:
			futures = [executor.submit(get_one_block_batch, dev_client_url, batch_lists[batch_id]) for batch_id in range(n_batches)]
            
			for future in cf.as_completed(futures):
				# get the result for the next completed task
				batch_blocks = future.result() # blocks
				for block in batch_blocks.keys():
					blocks[block] = batch_blocks[block]
		
	return blocks


# get the block and process it
def get_data_from_block(block_idx, block):

    data_vec = []

    program = "GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me"

    for t in block:
        transaction_message = t["transaction"]["message"]
        accounts = transaction_message["accountKeys"]

        for instruction in transaction_message["instructions"]:

            program_index = instruction["programIdIndex"]

            if (accounts[program_index] != program):
                continue
            
            if ("data" not in instruction.keys()):
                continue

            data = instruction["data"]
            decoded_data = base58.b58decode(data)

            try:
                args = message.parse(decoded_data)
            except:
                log_error("unable to parse data: " + str(decoded_data))
                continue

            if(not isinstance(args, message.enum.PushButton)):
                print("Have data but not a PushButton:", args)
                continue
            
            data_vec.append(args)

    return block_idx, data_vec	


# create the rows for the database from the block data
def create_rows_from_data(row_id_to_insert, block_id, data, rows_vec):

    if(len(data) == 0):
        new_row = (row_id_to_insert, block_id, "no_button", 0)
        log_db("adding row: " + str(new_row))
        rows_vec.append(new_row)
        row_id_to_insert += 1
    else:
        for i in range(len(data)):
            args = data[i]
            row_id = row_id_to_insert + i
            new_row = (row_id, block_id, str(args.button), args.amount)
            log_db("adding row: " + str(new_row))
            rows_vec.append(new_row)
			
        row_id_to_insert += len(data)
			
    return row_id_to_insert