# Программа с настоящим спрайтом
#v0 = 10  # X
v1 = 15  # Y

clear() # type: ignore

while True:
    v0 = 10
    #draw_sprite(v0, v1, 5)  # Рисуем 5-байтовый спрайт (цифра)
    print(v0, v1, 'B')
    v0 = 15
    print(v0, v1, 'A')
    v0 = 20
    print(v0, v1, '0')
    v0 = 25
    print(v0, v1, 'B')
    v0 = 30
    print(v0, v1, 'A')
    v0 = 35
    print(v0, v1, 'B')
