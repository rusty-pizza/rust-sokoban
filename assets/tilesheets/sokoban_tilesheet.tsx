<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.5" tiledversion="1.7.2" name="sokoban_tilesheet@2" tilewidth="128" tileheight="128" tilecount="104" columns="13" objectalignment="topleft">
 <properties>
  <property name="player_down" type="int" value="52"/>
  <property name="player_left" type="int" value="78"/>
  <property name="player_right" type="int" value="81"/>
  <property name="player_up" type="int" value="55"/>
 </properties>
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
 <tile id="39" type="goal">
  <animation>
   <frame tileid="39" duration="500"/>
   <frame tileid="13" duration="500"/>
  </animation>
 </tile>
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
 <tile id="52">
  <animation>
   <frame tileid="52" duration="500"/>
   <frame tileid="53" duration="500"/>
   <frame tileid="52" duration="500"/>
   <frame tileid="54" duration="500"/>
  </animation>
 </tile>
 <tile id="55">
  <animation>
   <frame tileid="55" duration="500"/>
   <frame tileid="56" duration="500"/>
   <frame tileid="55" duration="500"/>
   <frame tileid="57" duration="500"/>
  </animation>
 </tile>
 <tile id="72" type="spawn"/>
 <tile id="78">
  <animation>
   <frame tileid="78" duration="500"/>
   <frame tileid="79" duration="500"/>
   <frame tileid="78" duration="500"/>
   <frame tileid="80" duration="500"/>
  </animation>
 </tile>
 <tile id="81">
  <animation>
   <frame tileid="81" duration="500"/>
   <frame tileid="82" duration="500"/>
   <frame tileid="81" duration="500"/>
   <frame tileid="83" duration="500"/>
  </animation>
 </tile>
 <tile id="84" type="solid"/>
 <tile id="85" type="solid"/>
 <tile id="86" type="solid"/>
 <tile id="87" type="solid"/>
</tileset>
