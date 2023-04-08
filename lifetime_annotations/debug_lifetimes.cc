#include "debug_lifetimes.h"
#include <iostream>

// TODO remove this file

void debugLifetimes(std::string txt) { std::cout << txt << '\n'; }

void debugLifetimes(std::string txt1, std::string txt2) {
  std::cout << txt1 << ": " << txt2 << '\n';
}

void debugLifetimes(std::string txt, int i) {
  std::cout << txt << ": " << i << '\n';
}

void debugLifetimes(llvm::SmallVector<const clang::Expr*> vec) {
  for (const auto &el : vec) {
    el->dump();
  }

}

void debugLifetimes(llvm::SmallVector<const clang::Attr*> vec) {
  for (const auto &el : vec) {
    el->getNormalizedFullName();
  }
}

void debugLifetimes(llvm::SmallVector<std::string> vec) {
  for (const auto &el : vec) {
    debugLifetimes(el);
  }
}