import AutumnVestImg from '../assets/companions/autumn-vest.png';
import AutumnVestIdle from '../assets/companions/sprites/autumn-vest/idle.png';
import AutumnVestWalk from '../assets/companions/sprites/autumn-vest/walk.png';
import AutumnVestHello from '../assets/companions/sprites/autumn-vest/hello.png';
import AutumnVestThinking from '../assets/companions/sprites/autumn-vest/thinking.png';
import AutumnVestWorking from '../assets/companions/sprites/autumn-vest/working.png';
import AutumnVestCommand from '../assets/companions/sprites/autumn-vest/command.png';
import AutumnVestDone from '../assets/companions/sprites/autumn-vest/done.png';
import AutumnVestError from '../assets/companions/sprites/autumn-vest/error.png';
import BlueHoodieImg from '../assets/companions/blue-hoodie.png';
import BlueHoodieIdle from '../assets/companions/sprites/blue-hoodie/idle.png';
import BlueHoodieWalk from '../assets/companions/sprites/blue-hoodie/walk.png';
import BlueHoodieHello from '../assets/companions/sprites/blue-hoodie/hello.png';
import BlueHoodieThinking from '../assets/companions/sprites/blue-hoodie/thinking.png';
import BlueHoodieWorking from '../assets/companions/sprites/blue-hoodie/working.png';
import BlueHoodieCommand from '../assets/companions/sprites/blue-hoodie/command.png';
import BlueHoodieDone from '../assets/companions/sprites/blue-hoodie/done.png';
import BlueHoodieError from '../assets/companions/sprites/blue-hoodie/error.png';
import GreenScoutImg from '../assets/companions/green-scout.png';
import GreenScoutIdle from '../assets/companions/sprites/green-scout/idle.png';
import GreenScoutWalk from '../assets/companions/sprites/green-scout/walk.png';
import GreenScoutHello from '../assets/companions/sprites/green-scout/hello.png';
import GreenScoutThinking from '../assets/companions/sprites/green-scout/thinking.png';
import GreenScoutWorking from '../assets/companions/sprites/green-scout/working.png';
import GreenScoutCommand from '../assets/companions/sprites/green-scout/command.png';
import GreenScoutDone from '../assets/companions/sprites/green-scout/done.png';
import GreenScoutError from '../assets/companions/sprites/green-scout/error.png';
import LavenderBearImg from '../assets/companions/lavender-bear.png';
import LavenderBearIdle from '../assets/companions/sprites/lavender-bear/idle.png';
import LavenderBearWalk from '../assets/companions/sprites/lavender-bear/walk.png';
import LavenderBearHello from '../assets/companions/sprites/lavender-bear/hello.png';
import LavenderBearThinking from '../assets/companions/sprites/lavender-bear/thinking.png';
import LavenderBearWorking from '../assets/companions/sprites/lavender-bear/working.png';
import LavenderBearCommand from '../assets/companions/sprites/lavender-bear/command.png';
import LavenderBearDone from '../assets/companions/sprites/lavender-bear/done.png';
import LavenderBearError from '../assets/companions/sprites/lavender-bear/error.png';
import PinkHoodImg from '../assets/companions/pink-hood.png';
import PinkHoodIdle from '../assets/companions/sprites/pink-hood/idle.png';
import PinkHoodWalk from '../assets/companions/sprites/pink-hood/walk.png';
import PinkHoodHello from '../assets/companions/sprites/pink-hood/hello.png';
import PinkHoodThinking from '../assets/companions/sprites/pink-hood/thinking.png';
import PinkHoodWorking from '../assets/companions/sprites/pink-hood/working.png';
import PinkHoodCommand from '../assets/companions/sprites/pink-hood/command.png';
import PinkHoodDone from '../assets/companions/sprites/pink-hood/done.png';
import PinkHoodError from '../assets/companions/sprites/pink-hood/error.png';
import RedBeanieImg from '../assets/companions/red-beanie.png';
import RedBeanieIdle from '../assets/companions/sprites/red-beanie/idle.png';
import RedBeanieWalk from '../assets/companions/sprites/red-beanie/walk.png';
import RedBeanieHello from '../assets/companions/sprites/red-beanie/hello.png';
import RedBeanieThinking from '../assets/companions/sprites/red-beanie/thinking.png';
import RedBeanieWorking from '../assets/companions/sprites/red-beanie/working.png';
import RedBeanieCommand from '../assets/companions/sprites/red-beanie/command.png';
import RedBeanieDone from '../assets/companions/sprites/red-beanie/done.png';
import RedBeanieError from '../assets/companions/sprites/red-beanie/error.png';
import TanExplorerImg from '../assets/companions/tan-explorer.png';
import TanExplorerIdle from '../assets/companions/sprites/tan-explorer/idle.png';
import TanExplorerWalk from '../assets/companions/sprites/tan-explorer/walk.png';
import TanExplorerHello from '../assets/companions/sprites/tan-explorer/hello.png';
import TanExplorerThinking from '../assets/companions/sprites/tan-explorer/thinking.png';
import TanExplorerWorking from '../assets/companions/sprites/tan-explorer/working.png';
import TanExplorerCommand from '../assets/companions/sprites/tan-explorer/command.png';
import TanExplorerDone from '../assets/companions/sprites/tan-explorer/done.png';
import TanExplorerError from '../assets/companions/sprites/tan-explorer/error.png';
import YellowRainImg from '../assets/companions/yellow-rain.png';
import YellowRainIdle from '../assets/companions/sprites/yellow-rain/idle.png';
import YellowRainWalk from '../assets/companions/sprites/yellow-rain/walk.png';
import YellowRainHello from '../assets/companions/sprites/yellow-rain/hello.png';
import YellowRainThinking from '../assets/companions/sprites/yellow-rain/thinking.png';
import YellowRainWorking from '../assets/companions/sprites/yellow-rain/working.png';
import YellowRainCommand from '../assets/companions/sprites/yellow-rain/command.png';
import YellowRainDone from '../assets/companions/sprites/yellow-rain/done.png';
import YellowRainError from '../assets/companions/sprites/yellow-rain/error.png';


export type CompanionAnimationName = 'idle' | 'walk' | 'hello' | 'thinking' | 'working' | 'command' | 'done' | 'error';

export type CompanionAnimationSheet = {
  src: string;
  frames: number;
  duration: number;
  width: number;
  height: number;
};

export type CompanionCharacter = {
  id: string;
  name: string;
  image: string;
  palette: string;
  sprites: Record<CompanionAnimationName, CompanionAnimationSheet>;
};

export const COMPANION_CHARACTER_STORAGE_KEY = 'lil-buddy-companion-character-v1';

export const companionCharacters: CompanionCharacter[] = [
  {
    id: 'autumn-vest',
    name: 'Autumn Vest',
    image: AutumnVestImg,
    palette: 'cozy autumn',
    sprites: {
      idle: { src: AutumnVestIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: AutumnVestWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: AutumnVestHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: AutumnVestThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: AutumnVestWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: AutumnVestCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: AutumnVestDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: AutumnVestError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'blue-hoodie',
    name: 'Blue Hoodie',
    image: BlueHoodieImg,
    palette: 'sky blue',
    sprites: {
      idle: { src: BlueHoodieIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: BlueHoodieWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: BlueHoodieHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: BlueHoodieThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: BlueHoodieWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: BlueHoodieCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: BlueHoodieDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: BlueHoodieError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'green-scout',
    name: 'Green Scout',
    image: GreenScoutImg,
    palette: 'forest green',
    sprites: {
      idle: { src: GreenScoutIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: GreenScoutWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: GreenScoutHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: GreenScoutThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: GreenScoutWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: GreenScoutCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: GreenScoutDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: GreenScoutError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'lavender-bear',
    name: 'Lavender Bear',
    image: LavenderBearImg,
    palette: 'lavender cream',
    sprites: {
      idle: { src: LavenderBearIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: LavenderBearWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: LavenderBearHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: LavenderBearThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: LavenderBearWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: LavenderBearCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: LavenderBearDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: LavenderBearError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'pink-hood',
    name: 'Pink Hood',
    image: PinkHoodImg,
    palette: 'soft rose',
    sprites: {
      idle: { src: PinkHoodIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: PinkHoodWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: PinkHoodHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: PinkHoodThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: PinkHoodWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: PinkHoodCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: PinkHoodDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: PinkHoodError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'red-beanie',
    name: 'Red Beanie',
    image: RedBeanieImg,
    palette: 'trail helper',
    sprites: {
      idle: { src: RedBeanieIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: RedBeanieWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: RedBeanieHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: RedBeanieThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: RedBeanieWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: RedBeanieCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: RedBeanieDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: RedBeanieError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'tan-explorer',
    name: 'Tan Explorer',
    image: TanExplorerImg,
    palette: 'warm explorer',
    sprites: {
      idle: { src: TanExplorerIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: TanExplorerWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: TanExplorerHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: TanExplorerThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: TanExplorerWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: TanExplorerCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: TanExplorerDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: TanExplorerError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },
  {
    id: 'yellow-rain',
    name: 'Yellow Rain',
    image: YellowRainImg,
    palette: 'sunny raincoat',
    sprites: {
      idle: { src: YellowRainIdle, frames: 8, duration: 1500, width: 192, height: 192 },
      walk: { src: YellowRainWalk, frames: 10, duration: 760, width: 192, height: 192 },
      hello: { src: YellowRainHello, frames: 8, duration: 900, width: 192, height: 192 },
      thinking: { src: YellowRainThinking, frames: 8, duration: 850, width: 192, height: 192 },
      working: { src: YellowRainWorking, frames: 8, duration: 680, width: 192, height: 192 },
      command: { src: YellowRainCommand, frames: 8, duration: 440, width: 192, height: 192 },
      done: { src: YellowRainDone, frames: 8, duration: 820, width: 192, height: 192 },
      error: { src: YellowRainError, frames: 8, duration: 360, width: 192, height: 192 },
    },
  },

];

export function getSavedCompanionCharacterId() {
  return localStorage.getItem(COMPANION_CHARACTER_STORAGE_KEY) ?? companionCharacters[0].id;
}

export function saveCompanionCharacterId(id: string) {
  const exists = companionCharacters.some((character) => character.id === id);
  localStorage.setItem(COMPANION_CHARACTER_STORAGE_KEY, exists ? id : companionCharacters[0].id);
  window.dispatchEvent(new CustomEvent('lilman:companion-character', { detail: exists ? id : companionCharacters[0].id }));
}

export function getCompanionCharacter(id?: string | null) {
  return companionCharacters.find((character) => character.id === id) ?? companionCharacters[0];
}
