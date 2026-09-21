import type { MacroModule } from '../types';

/** 線型代数のマクロ． */
const linearAlgebra: MacroModule = {
  macros: {
    rank: String.raw`\operatorname{rank}`,
  },
};

export default linearAlgebra;
