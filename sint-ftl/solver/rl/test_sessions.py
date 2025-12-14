import unittest
import sint_solver
import uuid

class TestSessionManagement(unittest.TestCase):
    def setUp(self):
        self.player_ids = ["P1", "P2"]
        self.seed = 12345

    def test_verify_solution_session_caching(self):
        """Verify that verify_solution correctly caches and resumes using session_id."""
        session_id = str(uuid.uuid4())
        
        # Round 1 actions (consumes all AP)
        round1 = [
            ("P1", "Move 0"), ("P1", "Move 5"), 
            ("P2", "Move 0"), ("P2", "Move 9")
        ]
        rounds = [round1]
        
        # First call: initializes cache
        result1 = sint_solver.verify_solution(self.player_ids, self.seed, rounds, session_id=session_id)
        self.assertIsNone(result1.get('error'), f"Error in result1: {result1.get('error')}")
        state1_seq = result1['final_state']['sequence_id']
        
        # Round 2 actions
        round2 = [("P1", "Bake"), ("P1", "Pass"), ("P2", "Pass")]
        rounds_extended = [round1, round2]
        
        # Second call: should use prefix matching to resume from state1
        result2 = sint_solver.verify_solution(self.player_ids, self.seed, rounds_extended, session_id=session_id)
        self.assertIsNone(result2.get('error'), f"Error in result2: {result2.get('error')}")
        self.assertGreater(result2['final_state']['sequence_id'], state1_seq)

    def test_session_prefix_mismatch_fallback(self):
        """Verify that verify_solution falls back to a full simulation if history doesn't match."""
        session_id = str(uuid.uuid4())
        
        round1 = [("P1", "Move 0"), ("P1", "Move 5"), ("P2", "Pass")]
        result1 = sint_solver.verify_solution(self.player_ids, self.seed, [round1], session_id=session_id)
        self.assertIsNone(result1.get('error'), f"Error in result1: {result1.get('error')}")
        
        # Call with different round 1 (prefix mismatch)
        round1_alt = [("P1", "Move 0"), ("P1", "Move 2"), ("P2", "Pass")]
        result = sint_solver.verify_solution(self.player_ids, self.seed, [round1_alt], session_id=session_id)
        
        self.assertIsNone(result.get('error'), f"Error in result alt: {result.get('error')}")
        # The result should be valid for round1_alt, proving it didn't use the stale cache incorrectly
        self.assertEqual(result['final_state']['players']['P1']['room_id'], 2)

    def test_verify_linear_state_passing(self):
        """Verify that verify_linear correctly handles explicit state passing (RL style)."""
        # Get initial state
        res_init = sint_solver.verify_linear(self.player_ids, self.seed, [])
        state = res_init['final_state']
        
        # Step 1: Move
        action = ("P1", {"type": "Move", "payload": {"to_room": 0}})
        res_step1 = sint_solver.verify_linear(self.player_ids, self.seed, [action], initial_state=state)
        self.assertTrue(res_step1['final_state'] is not None)
        self.assertEqual(res_step1['final_state']['players']['P1']['room_id'], 0)
        
        # Step 2: Move again from state 1
        state1 = res_step1['final_state']
        action2 = ("P1", {"type": "Move", "payload": {"to_room": 5}})
        res_step2 = sint_solver.verify_linear(self.player_ids, self.seed, [action2], initial_state=state1)
        self.assertEqual(res_step2['final_state']['players']['P1']['room_id'], 5)

    def test_multiple_concurrent_sessions(self):
        """Verify that multiple sessions with different IDs do not interfere."""
        session1 = "session_1"
        session2 = "session_2"
        
        round1_s1 = [("P1", "Move 0"), ("P1", "Move 5"), ("P2", "Pass")]
        round1_s2 = [("P1", "Move 0"), ("P1", "Move 9"), ("P2", "Pass")]
        
        # Initialize both sessions
        res1 = sint_solver.verify_solution(self.player_ids, self.seed, [round1_s1], session_id=session1)
        res2 = sint_solver.verify_solution(self.player_ids, self.seed, [round1_s2], session_id=session2)
        
        self.assertEqual(res1['final_state']['players']['P1']['room_id'], 5)
        self.assertEqual(res2['final_state']['players']['P1']['room_id'], 9)
        
        # Resume both sessions
        round2 = [("P1", "Pass"), ("P2", "Pass")]
        
        res1_next = sint_solver.verify_solution(self.player_ids, self.seed, [round1_s1, round2], session_id=session1)
        res2_next = sint_solver.verify_solution(self.player_ids, self.seed, [round1_s2, round2], session_id=session2)
        
        self.assertEqual(res1_next['final_state']['players']['P1']['room_id'], 5)
        self.assertEqual(res2_next['final_state']['players']['P1']['room_id'], 9)
        self.assertNotEqual(res1_next['final_state']['sequence_id'], res1['final_state']['sequence_id'])
        self.assertNotEqual(res2_next['final_state']['sequence_id'], res2['final_state']['sequence_id'])

if __name__ == '__main__':
    unittest.main()
