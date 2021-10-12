<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.5" tiledversion="1.7.2" name="sokoban_tilesheet@2" tilewidth="128" tileheight="128" tilecount="104" columns="13" objectalignment="topleft">
 <image source="../sprites/Tilesheet/sokoban_tilesheet@2.png" width="1664" height="1024"/>
 <tile id="6" type="crate">
  <properties>
   <property name="style" type="int" value="1"/>
  </properties>
  <animation>
   <frame tileid="6" duration="1000"/>
   <frame tileid="45" duration="1000"/>
   <frame tileid="19" duration="1000"/>
  </animation>
 </tile>
 <tile id="7">
  <properties>
   <property name="style" type="int" value="2"/>
  </properties>
  <animation>
   <frame tileid="7" duration="1000"/>
   <frame tileid="46" duration="1000"/>
   <frame tileid="20" duration="1000"/>
  </animation>
 </tile>
 <tile id="8">
  <properties>
   <property name="style" type="int" value="3"/>
  </properties>
  <animation>
   <frame tileid="8" duration="1000"/>
   <frame tileid="47" duration="1000"/>
   <frame tileid="21" duration="1000"/>
  </animation>
 </tile>
 <tile id="9">
  <properties>
   <property name="style" type="int" value="4"/>
  </properties>
  <animation>
   <frame tileid="9" duration="1000"/>
   <frame tileid="48" duration="1000"/>
   <frame tileid="22" duration="1000"/>
  </animation>
 </tile>
 <tile id="10">
  <properties>
   <property name="style" type="int" value="5"/>
  </properties>
  <animation>
   <frame tileid="10" duration="1000"/>
   <frame tileid="49" duration="1000"/>
   <frame tileid="23" duration="1000"/>
  </animation>
 </tile>
 <tile id="11" type="hole"/>
 <tile id="39" type="goal"/>
 <tile id="40" type="goal">
  <properties>
   <property name="accepts" type="int" value="1"/>
  </properties>
 </tile>
 <tile id="41" type="goal">
  <properties>
   <property name="accepts" type="int" value="2"/>
  </properties>
 </tile>
 <tile id="42" type="goal">
  <properties>
   <property name="accepts" type="int" value="3"/>
  </properties>
 </tile>
 <tile id="43" type="goal">
  <properties>
   <property name="accepts" type="int" value="4"/>
  </properties>
 </tile>
 <tile id="44" type="goal">
  <properties>
   <property name="accepts" type="int" value="5"/>
  </properties>
 </tile>
 <tile id="72" type="spawn"/>
 <tile id="84" type="solid"/>
 <tile id="85" type="solid"/>
 <tile id="86" type="solid"/>
 <tile id="87" type="solid"/>
</tileset>
