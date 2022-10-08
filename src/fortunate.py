from fastapi import FastAPI
import node
from pool import Pool
import pool
from blockstorageserver import BlockStorageServer
from multiprocessing import Process
from time import sleep


def _runnode(port):
    _, _, nodebackend = node.create_node(port)
    nodebackend.open()


def _runpool():
    poolbackend = pool.create_poolbackend()
    poolbackend.ready()


def _runblockstorageserver():
    bss = BlockStorageServer()
    bss.app()


if __name__ == "__main__":
    blockserverproc = Process(target=_runblockstorageserver)
    blockserverproc.start()
    sleep(1.5)

    p0 = Process(target=_runnode, args=(5050,))
    p0.start()

    p1 = Process(target=_runnode, args=(5051,))
    p1.start()

    sleep(1.5)
    _runpool()