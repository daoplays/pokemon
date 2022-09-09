import time
import numpy as np
from pyboy import PyBoy, WindowEvent
import random as random
from datetime import datetime
from os.path import exists
from sql_funcs import *


class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    
def print_red(string):
		print(bcolors.FAIL + string + bcolors.ENDC)
		
def print_blue(string):
	print(bcolors.OKBLUE + string + bcolors.ENDC)
    
buttons = ["Button.A()", "Button.B()", "Button.Left()", "Button.Right()", "Button.Up()", "Button.Down()", "Button.Start()", "Button.Select()"]
button_map = {}
for i in range(len(buttons)):
	button = buttons[i]
	button_map[button] = i   
	

frames_per_block = 10
rom_name = "rom_name"
def handle_button(py, button):

	push = None
	release = None
	if(button == "Button.A()"):
		push = WindowEvent.PRESS_BUTTON_A
		release = WindowEvent.RELEASE_BUTTON_A
	elif(button == "Button.B()"):
		push = WindowEvent.PRESS_BUTTON_B
		release = WindowEvent.RELEASE_BUTTON_B
	elif(button == "Button.Left()"):
		push = WindowEvent.PRESS_ARROW_LEFT
		release = WindowEvent.RELEASE_ARROW_LEFT
	elif(button == "Button.Right()"):
		push = WindowEvent.PRESS_ARROW_RIGHT
		release = WindowEvent.RELEASE_ARROW_RIGHT
	elif(button == "Button.Up()"):
		push = WindowEvent.PRESS_ARROW_UP
		release = WindowEvent.RELEASE_ARROW_UP
	elif(button == "Button.Down()"):
		push = WindowEvent.PRESS_ARROW_DOWN
		release = WindowEvent.RELEASE_ARROW_DOWN

	elif(button == "Button.Start()"):
		push = WindowEvent.PRESS_BUTTON_START
		release = WindowEvent.RELEASE_BUTTON_START
	elif(button == "Button.Select()"):
		push = WindowEvent.PRESS_BUTTON_SELECT
		release = WindowEvent.RELEASE_BUTTON_SELECT
		
	py.send_input(push)
	for i in range(5):
		py.tick()
	py.send_input(release)
	for i in range(frames_per_block - 5):
		py.tick() 
		
	
def handle_no_button(block_idx, py):
	print("no button pressed for block:", block_idx)
	for i in range(frames_per_block):
		py.tick() 
	
def choose_button_from_rows(block_idx, rows):
    weights = np.zeros(len(buttons))
    for row in rows:
        button = row[2]
        weight = row[3]
        button_idx = button_map[button]
        weights[button_idx] += weight
	
    weights /= np.sum(weights)
    c_dist = np.cumsum(weights)
    random.seed(int(block_idx))
    r = random.random()
	
    button_chosen = 0
    for p in range(len(c_dist)):
        if(c_dist[p] > r):
            button_chosen = p
            break
			
    #print(rows)
    print(block_idx, c_dist, r, button_chosen, buttons[button_chosen])
    return buttons[button_chosen]

	
# handles the range inclusive of start and end
def handle_block_range(conn, pyboy, start_block, end_block):


	rows = get_rows_for_block_range(conn, start_block, end_block)
	print("have " + str(len(rows)) + " rows")
	if(len(rows) == 0):
		return
		
	block_rows = {}
	current_block = -1
	for i in range(len(rows)):
		row = rows[i]
		next_block = row[1]
		if (next_block != current_block):
			block_rows[next_block] = []
			block_rows[next_block].append(row)
			current_block = next_block
		else:
			block_rows[next_block].append(row)
			

	blocks = np.sort(list(block_rows.keys()))

	pyboy.set_emulation_speed(len(blocks))

	for block in blocks:
		#print(block, block_rows[block])
		process_block_rows(block_rows[block], pyboy, block)

	pyboy.set_emulation_speed(1)

		
		
def process_block_rows(rows, pyboy, block_idx):

	if(rows[0][2] == "no_button"):
		handle_no_button(block_idx, pyboy)
		return
			
	pressed = choose_button_from_rows(block_idx, rows)
	handle_button(pyboy, pressed)
	
# initialises the game for testing
def init_game(pyboy):

	for i in range(2000):
		pyboy.tick()
		
	for i in range(100):
		pyboy.send_input(WindowEvent.PRESS_BUTTON_A)
		pyboy.tick()
		pyboy.send_input(WindowEvent.RELEASE_BUTTON_A)
		for i in range(50):
			pyboy.tick()


# checks if there is saved state and will use that, otherwise has to just go from the start of the database	
def skip_to_present(conn, pyboy):

	# first just check if anything exists
	start_block = get_first_block_id(conn)
	end_block = get_last_block_id(conn)
	
	if(end_block == -1):
		init_game(pyboy)
		return end_block
		
	if (exists("save_file.state") and exists("current_block.state")):
		update_state(conn, end_block)
		check_block = load_state(pyboy)
		if (check_block != end_block):
			print_red("something bad happened.. " + str(check_block) + " " + str(end_block))
			return -2
			
		return end_block


	print("slow skip from ", start_block, " to ", end_block)
	headless = PyBoy(rom_name, window_type="dummy")
	init_game(headless)
	handle_block_range(conn, headless, start_block, end_block)
	save_state(headless, end_block)
	headless.stop()
	check_block = load_state(pyboy)
	if (check_block != end_block):
		print_red("something bad happened.. " + str(check_block) + " " + str(end_block))
		return -2
	
	return end_block
	
def test_play_speed(pyboy, speed = 1, seconds = 60, iterations = 1, save=False):

	pyboy.set_emulation_speed(speed)
	fps = 60
	
	for r in range(iterations):
		start_time = datetime.utcnow()
		for i in range(seconds * fps):
			pyboy.tick()
			
		if(save):
			save_state(pyboy)
		
		end_time = datetime.utcnow()
		print("Time for iteration ", r, ": ", (end_time - start_time).total_seconds())
	
def save_state(pyboy, current_block):

	print_red("saving state, current block: " + str(current_block))
	save_file = open("save_file.state", "wb")
	pyboy.save_state(save_file)
	save_file.close()
	np.savetxt("current_block.state", np.array([current_block]))
	
def load_state(pyboy):
	print("loading_state")
	save_file = open("save_file.state", "rb")
	pyboy.load_state(save_file)
	save_file.close()
	current_block = int(np.loadtxt("current_block.state")+0)

	return current_block
	
def update_state(conn, end_block):

	print_red("updating state")
	headless = PyBoy(rom_name, window_type="dummy")
	saved_block = load_state(headless)
		
	if(end_block == saved_block):
		return

	print("skip from ", saved_block + 1, " to ", end_block)
	handle_block_range(conn, headless, saved_block + 1, end_block)
	save_state(headless, end_block)
	headless.stop()

pyboy = PyBoy(rom_name)
			
conn = create_connection()

current_block = skip_to_present(conn, pyboy)
last_save = datetime.utcnow()
while(True):
	end_block = get_last_block_id(conn)
	
	if(end_block == current_block):
		#time.sleep(0.05)
		for i in range(5):
			pyboy.tick()
		continue
	
	print("have block", end_block)
	start_block = current_block + 1
	if(start_block == -1):
		start_block = end_block

	
	print("start:", start_block, "end:", end_block)
	handle_block_range(conn, pyboy, start_block, end_block)
	
	current_block = end_block
	
	# check if we should save state
	current_time = datetime.utcnow()
	time_since_save = (current_time - last_save).total_seconds()
	if (time_since_save > 60 * 5):
		save_state(pyboy, current_block)
		last_save = current_time