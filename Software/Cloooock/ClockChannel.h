//Header file for ClockChannel
#ifndef CLOCKCHANNEL_H
#define CLOCKCHANNEL_H

class ClockChannel {
public:
  ClockChannel(int channelPin, int ledPin, int division);
  void update();
  void assignDivision(int newDivision);
  void printDebug();
  void restart(unsigned long startTime);

private:
  int _division;
  int _value;
  int _ledPin;
  int _channelPin;
  int _hits; //reset at the start of every bar.
  int _barCount;
  int *_intervals;
  unsigned long _startTime;
};
#endif
