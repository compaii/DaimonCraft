scoreboard objectives add dc_x dummy
scoreboard objectives add dc_y dummy
scoreboard objectives add dc_z dummy
gamerule keep_inventory true
worldborder set @WORLD_BORDER@
schedule function daimoncraft:tick 1s replace
