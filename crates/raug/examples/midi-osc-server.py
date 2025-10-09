#!/usr/bin/env python3
import argparse
import logging
import sys
import time
import pygame
import pygame.midi
from pythonosc import udp_client
from pythonosc import dispatcher

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("midi-osc-server")


def print_handler(address, *args):
    logger.info(f"Received message at {address}: {args}")


def main():
    parser = argparse.ArgumentParser(description="MIDI to OSC server")
    parser.add_argument("--ip", default="localhost", help="The IP to listen on.")
    parser.add_argument("--port", type=int, default=8000, help="The port to listen on.")
    parser.add_argument("--client-ip", default="localhost", help="The client IP to send messages to.")
    parser.add_argument("--client-port", type=int, default=9000, help="The client port to send messages to.")
    parser.add_argument("--midi-device", type=int, default=None, help="The MIDI input device ID to use.")
    parser.add_argument("--list-midi-devices", action="store_true", help="List available MIDI input devices and exit.")
    args = parser.parse_args()

    pygame.midi.init()

    if args.list_midi_devices:
        for i in range(pygame.midi.get_count()):
            info = pygame.midi.get_device_info(i)
            interf, name, is_input, is_output, opened = info
            if is_input:
                logger.info(f"ID {i}: Interface: {interf.decode()}, Name: {name.decode()}, Opened: {opened}")
        pygame.midi.quit()
        sys.exit(0)

    disp = dispatcher.Dispatcher()
    disp.map("/print", print_handler)

    if args.midi_device is None:
        input_id = pygame.midi.get_default_input_id()
        if input_id == -1:
            logger.error("No MIDI input device found.")
            sys.exit(1)
    else:
        input_id = args.midi_device

    midi_input = pygame.midi.Input(input_id)
    logger.info(f"Using MIDI input device ID {input_id}")

    osc_client = udp_client.SimpleUDPClient(args.client_ip, args.client_port)

    try:

        while True:
            if midi_input.poll():
                midi_events = midi_input.read(10)
                for event in midi_events:
                    data, _ = event
                    status, note, velocity, _ = data  # type: ignore
                    osc_client.send_message("/midi", [status, note, velocity])
                    logger.info(f"/midi {status} {note} {velocity}")

                    match status & 0xF0:
                        case 0x90 if velocity > 0:
                            osc_client.send_message("/note_on", [note, velocity])
                            logger.info(f"/note_on {note} {velocity}")
                        case 0x80 | 0x90:
                            osc_client.send_message("/note_off", [note, velocity])
                            logger.info(f"/note_off {note} {velocity}")
                        case 0xB0:
                            osc_client.send_message("/control_change", [note, velocity])
                            logger.info(f"/control_change {note} {velocity}")
                        case 0xE0:
                            value = (velocity << 7) | note
                            osc_client.send_message("/pitch_bend", [value])
                            logger.info(f"/pitch_bend {value}")

            time.sleep(0.01)
    except KeyboardInterrupt:
        logger.info("Shutting down server.")
    finally:
        midi_input.close()
        pygame.midi.quit()
        logger.info("Server closed.")


if __name__ == "__main__":
    main()
