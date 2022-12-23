#include "ClockChannel.h"

uint8_t zero = 0b011000000;
uint8_t one = 0b11111001;
uint8_t two = 0b10100100;
uint8_t three = 0b10110000;
uint8_t four = 0b10011001;
uint8_t five = 0b10010010;
uint8_t six = 0b10000010;
uint8_t seven = 0b11111000;
uint8_t eight = 0B10000000;
uint8_t nine = 0b10010000;

void writeNumber(int n);
void readEncoder();
void setBPM(int _bpm);

uint8_t digits[] = {zero, one, two, three, four, five, six, seven, eight, nine};

int registerClock = 5;
int data = 6;
int latch = 4;

int encoderClock = A7;
int encoderDT = A6;
int encoderButton = 2;


int led0pin = A0;
int led1pin = A1;
int led2pin = A2;
int led3pin = A3;

int channel0pin = 7;
int channel1pin = 10;
int channel2pin = 8;
int channel3pin = 9;

ClockChannel channel0(channel0pin, led0pin, 2);
ClockChannel channel1(channel1pin, led1pin, 4);
ClockChannel channel2(channel2pin, led2pin, 8);
ClockChannel channel3(channel3pin, led3pin, 16);


unsigned long bpm = 120;  //beatsPerMinute
unsigned long previousBPM = 120;
unsigned long barLengthMillisecond;
int currentClockState;
int previousClockState;
unsigned long millisSinceButtonPress = 0;
unsigned long startTime;


void setup() {
  // put your setup code here, to run once:
  pinMode(registerClock, OUTPUT);
  pinMode(data, OUTPUT);
  pinMode(latch, OUTPUT);

  pinMode(led0pin, OUTPUT);
  pinMode(led1pin, OUTPUT);
  pinMode(led2pin, OUTPUT);
  pinMode(led3pin, OUTPUT);

  pinMode(encoderClock, INPUT);
  pinMode(encoderDT, INPUT);
  pinMode(encoderButton, INPUT);


  previousClockState = digitalRead(encoderClock);
  startTime = millis();
  setBPM(bpm);
  channel0.assignDivision(2);
  channel1.assignDivision(4);
  channel2.assignDivision(8);
  channel3.assignDivision(16);

  Serial.begin(9600);
}



void loop() {
  readEncoder();
  writeNumber(bpm);
  channel0.update();
  channel1.update();
  channel2.update();
  channel3.update();

  //channel3.printDebug();
}

void readEncoder() {
  currentClockState = analogRead(encoderClock) / 1000;
  if (currentClockState != previousClockState && currentClockState == 0) {

    // If the DT state is different than the CLK state then
    // the encoder is rotating CCW so decrement
    if (analogRead(encoderDT) / 1000 != currentClockState) {
      bpm--;
    } else {
      // Encoder is rotating CW so increment
      bpm++;
    }
    if (bpm > 1000) {
      bpm = 0;
    }
  }
  previousClockState = currentClockState;
}

void writeNumber(int n) {
  digitalWrite(latch, LOW);
  shiftOut(data, registerClock, MSBFIRST, 0b00000001);
  shiftOut(data, registerClock, MSBFIRST, digits[n / 100 % 10]);
  digitalWrite(latch, HIGH);
  delay(1);
  digitalWrite(latch, LOW);
  shiftOut(data, registerClock, MSBFIRST, 0B00000010);
  shiftOut(data, registerClock, MSBFIRST, digits[n / 10 % 10]);
  digitalWrite(latch, HIGH);
  delay(1);
  digitalWrite(latch, LOW);
  shiftOut(data, registerClock, MSBFIRST, 0B00000100);
  shiftOut(data, registerClock, MSBFIRST, digits[n / 1 % 10]);
  digitalWrite(latch, HIGH);
  delay(1);
  if (bpm != previousBPM) {
    startTime = millis();
    setBPM(bpm);
    unsigned long start = millis();
    channel0.restart(startTime);
    channel1.restart(startTime);
    channel2.restart(startTime);
    channel3.restart(startTime);
  }
  previousBPM = bpm;
}

void setBPM(int _bpm) {
  barLengthMillisecond = 1000000 / long((float(bpm) / 60.0) * 1000) * 4;
}