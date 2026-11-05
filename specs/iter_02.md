# Итерация 2: Обратный отсчёт времени на уровне + экран победы/поражения с результатом

## Заголовок
Добавить лимит времени на уровень (убывающий таймер) и полноэкранный overlay-экран победы/поражения, показывающий итоги (ходы, время, уровень), чтобы у переноса сундука появилась ставка и выигрыш/проигрыш ощущались событием.

## Мотивация
Сейчас нет никакой причины торопиться или беречь здоровье: игрок может бесконечно блуждать вслепую, а проигрыш/победа — это просто текстовая строка вверху, игра замирает без внятного итога. Это делает партию тягучей и обезличенной: пройти «красиво» и «кое-как» — одно и то же.

Обратный отсчёт даёт: (1) причину нести сундук быстро — игрок торопится и рискует, вскрывается наказание от медведей/ловушек; (2) понятную метрику результата через счётчик ходов. Overlay экран победы/поражения превращает конец партии в заметное событие, показывает затраченные ходы/время и сразу предлагает переиграть. Обе механики — чистый JS, один файл, не раскрывают карту дальше радиуса 1, умеренно по объёму.

## Конкретное ТЗ

### 1. Таймер уровня
- В блок «Состояние» добавить переменные:
  ```js
  let timeLeft = 0;   // секунд осталось на текущий уровень
  let timerId = null; // id интервала
  ```
- Константа времени в начале блока «Состояние»:
  ```js
  const LEVEL_TIME = 90; // секунд на уровень (уровень 1)
  ```
- В `init()` после `moves = 0;` задать `timeLeft = LEVEL_TIME;` и, если `timerId` уже есть, `clearInterval(timerId)`.
- Новый интервал — запускать в конце `init()` (после `render()` нет, лучше до него, порядок не критичен):
  ```js
  timerId = setInterval(() => {
    if (over) return;
    timeLeft--;
    updateHUD();
    if (timeLeft <= 0) endTimeout();
  }, 1000);
  ```
  Если `timeLeft <= 0` — мягкая остановка: выставить `timeLeft = 0`, вызвать `endTimeout()`.

### 2. HUD — таймер
- В HTML в `.hud` после `<span>Компас: ...</span>` добавить:
  ```html
  <span>Время: <b id="hud-time">--</b></span>
  ```
- `const hudTime = document.getElementById('hud-time');`
- В `updateHUD()` добавить запись:
  ```js
  hudTime.textContent = (timeLeft > 999 ? 999 : timeLeft) + 'с';
  ```
  Простой текст «NNс» (без счётчика ровно на 60 в минуту — излишне).

### 3. Overlay экран победы/поражения
- В HTML перед `<div id="legend">` (или после `</button>` — где угодно в `body`) добавить блок:
  ```html
  <div id="overlay" hidden>
    <div id="ov-panel">
      <h2 id="ov-title"></h2>
      <div id="ov-stats"></div>
      <button id="ov-replay">Играть снова</button>
    </div>
  </div>
  ```
- В `<style>` добавить стили (тёмный полупрозрачный фон поверх всего + центрированная карточка):
  ```css
  #overlay{ position:fixed; inset:0; background:rgba(8,10,14,.78);
    display:flex; align-items:center; justify-content:center; z-index:10; }
  #overlay[hidden]{ display:none; }
  #ov-panel{ background:#1b1e26; border:1px solid #2e3340; border-radius:16px;
    padding:28px 40px; text-align:center; box-shadow:0 20px 70px rgba(0,0,0,.7); }
  #ov-title{ margin:0 0 14px; font-size:26px; letter-spacing:2px; }
  #ov-stats{ margin-bottom:20px; font-size:16px; color:#c3c7cf; line-height:1.7; }
  #ov-replay{ padding:10px 30px; font-size:15px; font-weight:700; border:none;
    border-radius:999px; cursor:pointer; color:#14161c;
    background:linear-gradient(135deg,#ffd166,#ff9f43); }
  #ov-replay:hover{ filter:brightness(1.08); }
  ```
- В JS получить элементы:
  ```js
  const overlay = document.getElementById('overlay');
  const ovTitle = document.getElementById('ov-title');
  const ovStats = document.getElementById('ov-stats');
  const ovReplay = document.getElementById('ov-replay');
  ```
- Функция показа (единая для победы/поражения):
  ```js
  function showOverlay(title, color, statsHtml){
    ovTitle.textContent = title;
    ovTitle.style.color = color;
    ovStats.innerHTML = statsHtml;
    overlay.hidden = false;
  }
  ```
- `ovReplay.addEventListener('click', () => { init(); render(); });` — `init()` сам сбросит `over`, заново начнёт таймер и скроет overlay (см. п.4).

### 4. Подключение к концовкам
- В `endWin()`: сохранить посимвольно в локальные переменные перед изменением `level`:
  ```js
  endWin(){
    over = true;
    if (timerId) { clearInterval(timerId); timerId = null; }
    const used = moves;          // зафиксировать до level++
    const timeSpent = LEVEL_TIME - timeLeft;
    level++;
    msg('🏆 ПОБЕДА! Ты вынес сундук из лабиринта.');
    info('Уровень ' + level + '! Сложность растёт. Нажми R, чтобы сыграть ещё.');
    showOverlay('🏆 ПОБЕДА', '#ffd166',
      'Уровень пройден за <b>' + used + '</b> ходов · ' + timeSpent + 'с' +
      '<br>Новый уровень: <b>' + level + '</b> — сложность растёт');
  }
  ```
- В `endLose()` — то же по образцу:
  ```js
  endLose(){
    over = true;
    if (timerId) { clearInterval(timerId); timerId = null; }
    const used = moves;
    const timeSpent = LEVEL_TIME - timeLeft;
    level = 1;
    msg('💀 Ты погиб. Игра окончена.');
    info('Нажми R, чтобы сыграть ещё.');
    showOverlay('💀 ПОРАЖЕНИЕ', '#e63946',
      'Достигнуто: <b>' + used + '</b> ходов · ' + timeSpent + 'с' +
      '<br>Уровень сброшен до 1');
  }
  ```
- Новая функция `endTimeout()` (проигрыш по времени):
  ```js
  function endTimeout(){
    if (over) return;
    over = true;
    if (timerId) { clearInterval(timerId); timerId = null; }
    carrying = false; // клад при проигрыше не переносится
    level = 1;
    msg('⏰ Время вышло!');
    info('Нажми R, чтобы сыграть ещё.');
    showOverlay('⏰ ВРЕМЯ ВЫШЛО', '#f4a261',
      'Достигнуто: <b>' + moves + '</b> ходов<br>Уровень сброшен до 1');
  }
  ```

### 5. Скрытие overlay при рестарте
- В `init()` в самом начале добавить `overlay.hidden = true;` (сбрасывает экран и при нажатии R, и при «Играть снова», и при победе при переходе на новый уровень). Это гарантирует, что overlay исчезает на новой партии.

## Что нельзя трогать (регрессионная защита)
- **Правило тумана** `visible(r,c)` и весь рендер видимости: карта по-прежнему видна только в радиусе 1, не больше.
- **Генерация**: `genWalls`, `ensureConnected`, `cellCount`, `reachable`, раскладка объектов в `init()` (`river/holes/arsenal/hospital/trap/bears`), прогрессия 4-го уровня.
- **Механики**: движение `tryMove`, река `riverJump`, дырки `nextHole`, арсенал/госпиталь, ловушки, медведи `bearTurn`, бомбы `bomb`/`aiming`, подбор сундука и выход.
- **HUD-блоки** здоровья/бомб/клада/компаса и `activeTarget()`, пульс-подсказка в `render()`, клавиши (стрелки/WASD/B/R) и кнопка «Заново». Существующая кнопка `#restart` продолжает работать как раньше.
- В `endWin/endLose` **не удалять** `level++`/`level=1` и текущие `msg/info` — они сохраняются, overlay добавляется рядом.

## Критерий приёмки
1. На старте партии в HUD виден «Время: 300с»; каждую секунду уменьшается на 1. Во время игры таймер тикает; на новых уровнях снова сбрасывается на 300с.
2. При победе появляется overlay-окно «🏆 ПОБЕДА» с числом ходов и затраченным временем; уровень в подзаголовке растёт. Под overlay карта и HUD закрыты (не кликаются), кнопка «Играть снова» начинает новую партию того же повышенного уровня, overlay исчезает.
3. При смерти (здоровье 0) — overlay «💀 ПОРАЖЕНИЕ» с метриками; при сбросе времени (300-й тик без победы) — overlay «⏰ ВРЕМЯ ВЫШЛО»; уровень сбрасывается до 1.
4. Клавиша R и старая кнопка «Заново» по-прежнему сбрасывают партию и убирают overlay; туман, карта, все механики работают как в итерации 1.
5. Таймер и overlay не показывают карту и не дают никакой информации о дальних клетках — радиус 1 соблюдён.

## Риски
- Таймер добавляет систему реального времени; важно `clearInterval` при каждой концовке и в `init()`, чтобы интервалы не накапливались (утечка). Universal case: повторный `init()` без очистки — главный риск; обработка описана в п.1 и п.5.
- `LEVEL_TIME=90` — стартовая калибровка; если карта покажется тесной/лёгкой, настройку можно поднять/опустить, не трогая логику.