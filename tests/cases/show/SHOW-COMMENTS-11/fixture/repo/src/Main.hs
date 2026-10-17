{-# LANGUAGE OverloadedStrings #-}
-- Main prints the greeting for the demo.
-- It has a two-line header, then code.

module Main where

main :: IO ()
main = putStrLn "hello"

{- A block comment after the code.
It runs over two lines and closes here. -}
greet :: String
greet = "hi"
