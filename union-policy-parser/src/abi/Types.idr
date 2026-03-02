||| UNION-POLICY-PARSER — ABI Type Definitions
|||
||| This module defines the Application Binary Interface for the policy
||| orchestration engine. It ensures that complex "Union Policies" are
||| handled with strict type-safety across the proof and execution layers.

module UNION_POLICY_PARSER.ABI.Types

import Data.Bits
import Data.So
import Data.Vect

%default total

--------------------------------------------------------------------------------
-- Platform Context
--------------------------------------------------------------------------------

||| Supported targets for policy verification.
public export
data Platform = Linux | Windows | MacOS | BSD | WASM

||| Resolves the target environment at compile time.
public export
thisPlatform : Platform
thisPlatform =
  %runElab do
    pure Linux

--------------------------------------------------------------------------------
-- Core Result Codes
--------------------------------------------------------------------------------

||| Formal outcome of a policy parsing or validation operation.
public export
data Result : Type where
  ||| Operation Successful
  Ok : Result
  ||| Operation Failed: Policy violation
  Error : Result
  ||| Malformed Policy: Syntax error in input
  InvalidParam : Result
  ||| System Error: Out of memory
  OutOfMemory : Result
  ||| Safety Error: Internal null pointer encountered
  NullPointer : Result

--------------------------------------------------------------------------------
-- Opaque Policy Handles
--------------------------------------------------------------------------------

||| Opaque handle to a Policy instance.
||| INVARIANT: The internal pointer is guaranteed to be non-null.
public export
data Handle : Type where
  MkHandle : (ptr : Bits64) -> {auto 0 nonNull : So (ptr /= 0)} -> Handle

||| Safe constructor for policy handles.
public export
createHandle : Bits64 -> Maybe Handle
createHandle 0 = Nothing
createHandle ptr = Just (MkHandle ptr)
