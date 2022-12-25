#include "ClockChannel.h"

void printArray(int array[], int size) {
  for (int i = 0; i < size; i++) {
    Serial.print(array[i]);
    Serial.print(" ");
  }
  Serial.println();
}

ClockChannel::ClockChannel(int channelPin, int ledPin, int division) {
  _channelPin = channelPin;
  _ledPin = ledPin;
  _division = division;
  _value = 0;
  _barCount = 0;
  _intervals = nullptr;
  _startTime = 0;
  _hits = 0;
}

void ClockChannel::restart(unsigned long startTime) {
  _startTime = startTime;
  _barCount = 0;
  _value = LOW;
  assignDivision(_division/2);
  _hits = 0;
}

void ClockChannel::printDebug() {
  Serial.print("Time Division: ");
  Serial.print(_division);
  Serial.println("");
  //Serial.println("Intervals: ");
  //printArray(_intervals, _division);

  Serial.print("Hits: ");
  Serial.print(_hits);
}

void ClockChannel::update() {
  unsigned long time = millis()-_startTime;
  //Serial.println(time % barLengthMillisecond)
  if (time % barLengthMillisecond >= _intervals[_hits] || _barCount<time/barLengthMillisecond || _hits >= _division) {
    
    if(_hits >= _division) {
      if(_barCount<time/barLengthMillisecond) {
        _barCount = time/barLengthMillisecond;
        _hits = 0;
      }
      
    }else if (_value == HIGH) {
      //printArray(_intervals, _division);
      _value = LOW;
      _hits ++;
    } else {
      _value = HIGH;
      _hits ++;
    }
    
  }
  digitalWrite(_channelPin, _value);
  digitalWrite(_ledPin, _value);
}

void ClockChannel::assignDivision(int newDivision) {
  if (intervals != nullptr) {
    free(_intervals);
  }
  _division = newDivision * 2;
  _intervals = (int *)malloc(_division * sizeof(int));
  int beatLength = barLengthMillisecond/_division;
  //Serial.println(beatLength);
  int reminder = barLengthMillisecond%_division;
  for(int i = 0; i< _division; i++) {
    _intervals[i] = i * beatLength;
    if(i % 2 == 0 && i/2 <= reminder) { // crude way to distribute the reminder or part of it evenly in the list.
      _intervals[i] ++;
    }
  }
}

int counter = 0;
int thresh = 1000/4;

loop {
  counter += 1;
  if (counter%250 ==) {
    counter = 0;
    changeOutput();
  }
}