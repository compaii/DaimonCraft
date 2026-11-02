execute as @a store result score @s dc_x run data get entity @s Pos[0] 1
execute as @a store result score @s dc_y run data get entity @s Pos[1] 1
execute as @a store result score @s dc_z run data get entity @s Pos[2] 1
execute as @a run title @s actionbar [{"text":"DaimonCraft  ","color":"green"},{"score":{"name":"@s","objective":"dc_x"},"color":"aqua"},{"text":" / "},{"score":{"name":"@s","objective":"dc_y"}},{"text":" / "},{"score":{"name":"@s","objective":"dc_z"}}]
schedule function daimoncraft:tick 1s replace
