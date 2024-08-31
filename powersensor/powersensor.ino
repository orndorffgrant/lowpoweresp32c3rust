/*
    SEN0291 Gravity: I2C Digital Wattmeter
    The module is connected in series between the power supply and the load to read
    the voltage, current and power.
    The module has four I2C, these addresses are:
      INA219_I2C_ADDRESS1 0x40    A0 = 0 A1 = 0
      INA219_I2C_ADDRESS2 0x41    A0 = 1 A1 = 0
      INA219_I2C_ADDRESS3 0x44    A0 = 0 A1 = 1
      INA219_I2C_ADDRESS4 0x45    A0 = 1 A1 = 1
    Copyright [DFRobot](http://www.dfrobot.com), 2016
    Copyright GNU Lesser General Public License
    version V0.1
    date 2019-2-27

    Modified for use in class project by Grant Orndorff
    date 2024-04-18
*/
#include <Wire.h>
#include "DFRobot_INA219.h"

DFRobot_INA219_IIC ina219(&Wire, INA219_I2C_ADDRESS4);

// Revise the following two paramters according to actual reading of the INA219 and the multimeter
// for linearly calibration
float ina219Reading_mA = 25;
float extMeterReading_mA = 26;

void setup(void) {
  Serial.begin(115200);
  while (!Serial)
    ;

  Serial.println();
  while (ina219.begin() != true) {
    Serial.println("INA219 begin faild");
    delay(2000);
  }
  ina219.linearCalibrate(ina219Reading_mA, extMeterReading_mA);
  Serial.println();
}

void loop(void) {
  Serial.println(ina219.getCurrent_mA(), 1);
  delay(100);
}