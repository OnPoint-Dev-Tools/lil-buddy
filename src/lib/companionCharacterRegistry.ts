import idle01 from '../assets/companions/tan-explorer-v2/idle/01.png';
import idle02 from '../assets/companions/tan-explorer-v2/idle/02.png';
import idle03 from '../assets/companions/tan-explorer-v2/idle/03.png';
import idle04 from '../assets/companions/tan-explorer-v2/idle/04.png';
import idle05 from '../assets/companions/tan-explorer-v2/idle/05.png';
import idle06 from '../assets/companions/tan-explorer-v2/idle/06.png';

import hello01 from '../assets/companions/tan-explorer-v2/hello/01.png';
import hello02 from '../assets/companions/tan-explorer-v2/hello/02.png';
import hello03 from '../assets/companions/tan-explorer-v2/hello/03.png';
import hello04 from '../assets/companions/tan-explorer-v2/hello/04.png';
import hello05 from '../assets/companions/tan-explorer-v2/hello/05.png';
import hello06 from '../assets/companions/tan-explorer-v2/hello/06.png';

import walkRight01 from '../assets/companions/tan-explorer-v2/walk-right/01.png';
import walkRight02 from '../assets/companions/tan-explorer-v2/walk-right/02.png';
import walkRight03 from '../assets/companions/tan-explorer-v2/walk-right/03.png';
import walkRight04 from '../assets/companions/tan-explorer-v2/walk-right/04.png';
import walkRight05 from '../assets/companions/tan-explorer-v2/walk-right/05.png';
import walkRight06 from '../assets/companions/tan-explorer-v2/walk-right/06.png';
import walkRight07 from '../assets/companions/tan-explorer-v2/walk-right/07.png';
import walkRight08 from '../assets/companions/tan-explorer-v2/walk-right/08.png';

import walkLeft01 from '../assets/companions/tan-explorer-v2/walk-left/01.png';
import walkLeft02 from '../assets/companions/tan-explorer-v2/walk-left/02.png';
import walkLeft03 from '../assets/companions/tan-explorer-v2/walk-left/03.png';
import walkLeft04 from '../assets/companions/tan-explorer-v2/walk-left/04.png';
import walkLeft05 from '../assets/companions/tan-explorer-v2/walk-left/05.png';
import walkLeft06 from '../assets/companions/tan-explorer-v2/walk-left/06.png';
import walkLeft07 from '../assets/companions/tan-explorer-v2/walk-left/07.png';
import walkLeft08 from '../assets/companions/tan-explorer-v2/walk-left/08.png';

import thinking01 from '../assets/companions/tan-explorer-v2/thinking/01.png';
import thinking02 from '../assets/companions/tan-explorer-v2/thinking/02.png';
import thinking03 from '../assets/companions/tan-explorer-v2/thinking/03.png';
import thinking04 from '../assets/companions/tan-explorer-v2/thinking/04.png';
import thinking05 from '../assets/companions/tan-explorer-v2/thinking/05.png';
import thinking06 from '../assets/companions/tan-explorer-v2/thinking/06.png';

import working01 from '../assets/companions/tan-explorer-v2/working/01.png';
import working02 from '../assets/companions/tan-explorer-v2/working/02.png';
import working03 from '../assets/companions/tan-explorer-v2/working/03.png';
import working04 from '../assets/companions/tan-explorer-v2/working/04.png';
import working05 from '../assets/companions/tan-explorer-v2/working/05.png';
import working06 from '../assets/companions/tan-explorer-v2/working/06.png';

import command01 from '../assets/companions/tan-explorer-v2/command/01.png';
import command02 from '../assets/companions/tan-explorer-v2/command/02.png';
import command03 from '../assets/companions/tan-explorer-v2/command/03.png';
import command04 from '../assets/companions/tan-explorer-v2/command/04.png';
import command05 from '../assets/companions/tan-explorer-v2/command/05.png';
import command06 from '../assets/companions/tan-explorer-v2/command/06.png';

import done01 from '../assets/companions/tan-explorer-v2/done/01.png';
import done02 from '../assets/companions/tan-explorer-v2/done/02.png';
import done03 from '../assets/companions/tan-explorer-v2/done/03.png';
import done04 from '../assets/companions/tan-explorer-v2/done/04.png';
import done05 from '../assets/companions/tan-explorer-v2/done/05.png';
import done06 from '../assets/companions/tan-explorer-v2/done/06.png';

import error01 from '../assets/companions/tan-explorer-v2/error/01.png';
import error02 from '../assets/companions/tan-explorer-v2/error/02.png';
import error03 from '../assets/companions/tan-explorer-v2/error/03.png';
import error04 from '../assets/companions/tan-explorer-v2/error/04.png';
import error05 from '../assets/companions/tan-explorer-v2/error/05.png';
import error06 from '../assets/companions/tan-explorer-v2/error/06.png';

export type CompanionAnimationMood =
  | 'hello'
  | 'idle'
  | 'walk-left'
  | 'walk-right'
  | 'thinking'
  | 'working'
  | 'command'
  | 'done'
  | 'error';

export type CompanionAnimationClip = {
  frames: string[];
  fps: number;
  loop: boolean;
};

export type CompanionAnimationSet = Record<CompanionAnimationMood, CompanionAnimationClip>;

export type CompanionCharacterDefinition = {
  id: string;
  name: string;
  label: string;
  animations: CompanionAnimationSet;
};

const idle = [idle01, idle02, idle03, idle04, idle05, idle06];
const hello = [hello01, hello02, hello03, hello04, hello05, hello06];
const walkRight = [walkRight01, walkRight02, walkRight03, walkRight04, walkRight05, walkRight06, walkRight07, walkRight08];
const walkLeft = [walkLeft01, walkLeft02, walkLeft03, walkLeft04, walkLeft05, walkLeft06, walkLeft07, walkLeft08];
const thinking = [thinking01, thinking02, thinking03, thinking04, thinking05, thinking06];
const working = [working01, working02, working03, working04, working05, working06];
const command = [command01, command02, command03, command04, command05, command06];
const done = [done01, done02, done03, done04, done05, done06];
const error = [error01, error02, error03, error04, error05, error06];

export const companionCharactersV2: CompanionCharacterDefinition[] = [
  {
    id: 'tan-explorer-v2',
    name: 'Tan Explorer',
    label: 'ready',
    animations: {
      hello: { frames: hello, fps: 5, loop: false },
      idle: { frames: idle, fps: 4, loop: true },
      'walk-left': { frames: walkLeft, fps: 6, loop: true },
      'walk-right': { frames: walkRight, fps: 6, loop: true },
      thinking: { frames: thinking, fps: 4, loop: true },
      working: { frames: working, fps: 5, loop: true },
      command: { frames: command, fps: 5, loop: false },
      done: { frames: done, fps: 5, loop: false },
      error: { frames: error, fps: 5, loop: false },
    },
  },
];

export function getPrimaryCompanionCharacter() {
  return companionCharactersV2[0];
}
