#!/usr/bin/python

import solana.system_program as sp
from solana.publickey import PublicKey
from solana.transaction import Transaction, TransactionInstruction, AccountMeta
from solana.rpc.types import TxOpts
from solana.rpc.api import Client
from borsh_construct import Enum, CStruct, U64, U8
import spl.token.instructions as spl_token_instructions

import solana as sol
import sys

import argparse
import numpy as np
from spl.token.constants import ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID

from PyQt6.QtWidgets import (QWidget, QGridLayout, QPushButton, QToolButton, QApplication, QSizePolicy)
from PyQt6 import QtCore

PROGRAM_KEY = PublicKey("GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me")
MINT_KEY = PublicKey("6PRgpKnwT9xgGF7cgS7ZMkPBeQmd5mdS97eg26ir8Kki")

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

DPPInstructions = Enum(
"CreateDataAccount" / CStruct("amount" / U64),
"PushButton" / CStruct("button" / U8, "amount" / U64),
"PlaceBid"/ CStruct("amount_charity" / U64, "amount_dao" / U64, "charity" / charity_type),
"SelectWinners",
"SendTokens",
enum_name="DPPInstruction", 
)        
       
quick_node_dev = "https://api.mainnet-beta.solana.com"

dev_client = Client(quick_node_dev)

 
class Controller(QWidget):

    def __init__(self, wallet):
        super().__init__()

        self.wallet = wallet
        self.initUI()

    
        
    def load_key(self, filename):
        skey = open(filename).readlines()[0][1:-1].split(",")
        int_key = []
        for element in skey:
            int_key.append(int(element))
            
        owner=sol.keypair.Keypair.from_secret_key(bytes(int_key)[:32])
        
        return owner
            
    def send_transaction(self, dev_client, instructions):

        wallet = self.load_key(self.wallet)

        blockhash = dev_client.get_recent_blockhash()['result']['value']['blockhash']
        txn = Transaction(recent_blockhash=blockhash, fee_payer=wallet.public_key)

        for idx in instructions:
            txn.add(idx)

        txn.sign(wallet)

        response = dev_client.send_transaction(
            txn,
            wallet,
            opts=TxOpts(skip_preflight=True, skip_confirmation=True)
        )

        print(response)

    def get_press_button_idx(self, button, amount):

        wallet = self.load_key(self.wallet)
        program_data_account, data_bump = PublicKey.find_program_address([bytes("token_account", encoding="utf-8")], PROGRAM_KEY)
        program_token_account = spl_token_instructions.get_associated_token_address(program_data_account, MINT_KEY)
        user_token_account = spl_token_instructions.get_associated_token_address(wallet.public_key, MINT_KEY)

        amount = np.uint64(amount)
        button = np.uint8(button)
        instruction = TransactionInstruction(
            program_id = PROGRAM_KEY,
            data = DPPInstructions.build(DPPInstructions.enum.PushButton(button = button, amount = amount)),
            keys = [
                AccountMeta(pubkey=wallet.public_key, is_signer=True, is_writable=True),
                AccountMeta(pubkey=user_token_account, is_signer=False, is_writable=True),
                AccountMeta(pubkey=program_token_account, is_signer=False, is_writable=True),
                AccountMeta(pubkey=MINT_KEY, is_signer=False, is_writable=False),
                AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False)
                ]
        )

        return instruction

    def press_button(self, button, amount):

        idx = self.get_press_button_idx(button, amount)
        self.send_transaction(dev_client, [idx])
        

    def initUI(self):

        
        grid = QGridLayout()
        self.setLayout(grid)

        names = ['A', 'B', 'U', 'D',  'L', 'R',   'Start', 'Select']
        positions = [(1,3), (1,4), (0,1), (2,1),  (1,0), (1,2),    (4,0), (4,2)]
        buttons = [0,1,2,3,4,5,6,7]

        # A button
        button = QPushButton(names[0])
        button.setFixedSize(50,50)
        button.setStyleSheet("background-color : crimson")
        button.clicked.connect(lambda : self.press_button(buttons[0], 1))
        grid.addWidget(button, *positions[0])

        # B Button
        button = QPushButton(names[1])
        button.setFixedSize(50,50)
        button.setStyleSheet("background-color : crimson")
        button.clicked.connect(lambda : self.press_button(buttons[1], 1))
        grid.addWidget(button, *positions[1])

        # Up
        button = QToolButton()
        button.setFixedSize(50,50)
        button.setArrowType(QtCore.Qt.ArrowType.UpArrow)
        button.clicked.connect(lambda : self.press_button(buttons[2], 1))
        grid.addWidget(button, *positions[2])

        # Down
        button = QToolButton()
        button.setFixedSize(50,50)
        button.setArrowType(QtCore.Qt.ArrowType.DownArrow)
        button.clicked.connect(lambda : self.press_button(buttons[3], 1))
        grid.addWidget(button, *positions[3])

        # Left
        button = QToolButton()    
        button.setFixedSize(50,50)
        button.setArrowType(QtCore.Qt.ArrowType.LeftArrow)
        button.clicked.connect(lambda : self.press_button(buttons[4], 1))
        grid.addWidget(button, *positions[4])

        # Right
        button = QToolButton()
        button.setFixedSize(50,50)
        button.setArrowType(QtCore.Qt.ArrowType.RightArrow)
        button.clicked.connect(lambda : self.press_button(buttons[5], 1))
        grid.addWidget(button, *positions[5])

        # Start
        button = QPushButton(names[6])
        button.setFixedSize(50,20)
        button.setStyleSheet("background-color : grey")
        button.clicked.connect(lambda : self.press_button(buttons[6], 1))
        grid.addWidget(button, *positions[6])

        # Select
        button = QPushButton(names[7])
        button.setFixedSize(50,20)
        button.setStyleSheet("background-color : grey")
        button.clicked.connect(lambda : self.press_button(buttons[7], 1))
        grid.addWidget(button, *positions[7])

        self.move(300, 150)
        self.setWindowTitle('DaoPlays Pokemon Controller')
        self.show()


def main():

    parser = argparse.ArgumentParser()

    parser.add_argument("--wallet", help= "location of paper wallet for making moves in game", required=True)

    args = parser.parse_args()

    app = QApplication(sys.argv)
    ex = Controller(args.wallet)
    sys.exit(app.exec())


if __name__ == '__main__':
    main()