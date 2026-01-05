// API utilities for REST endpoints

import type { CreateSessionResponse, Customer } from '../types';

const API_BASE = '/api';

/**
 * Create a new conversation session
 */
export async function createSession(): Promise<CreateSessionResponse> {
  const response = await fetch(`${API_BASE}/sessions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to create session: ${response.status}`);
  }

  return response.json();
}

/**
 * Get session information
 */
export async function getSession(sessionId: string) {
  const response = await fetch(`${API_BASE}/sessions/${sessionId}`);

  if (!response.ok) {
    throw new Error(`Failed to get session: ${response.status}`);
  }

  return response.json();
}

/**
 * Delete a session
 */
export async function deleteSession(sessionId: string): Promise<void> {
  const response = await fetch(`${API_BASE}/sessions/${sessionId}`, {
    method: 'DELETE',
  });

  if (!response.ok) {
    throw new Error(`Failed to delete session: ${response.status}`);
  }
}

/**
 * Check API health
 */
export async function checkHealth(): Promise<boolean> {
  try {
    const response = await fetch('/health');
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Check API readiness
 */
export async function checkReady(): Promise<boolean> {
  try {
    const response = await fetch('/ready');
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Get WebSocket URL for a session
 */
export function getWebSocketUrl(sessionId: string): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}/ws/${sessionId}`;
}

/**
 * Fetch customer list (mock data if API unavailable)
 */
export async function fetchCustomers(): Promise<Customer[]> {
  try {
    const response = await fetch(`${API_BASE}/customers`);
    if (response.ok) {
      const data = await response.json();
      return data.customers;
    }
  } catch (error) {
    console.warn('Customer API unavailable, using mock data');
  }

  // Return mock data
  return [
    {
      id: 'C001',
      name: 'Rajesh Kumar',
      language: 'hi',
      segment: 'high_value',
      current_provider: 'muthoot',
      estimated_outstanding: 800000,
      estimated_rate: 18,
      city: 'Mumbai',
    },
    {
      id: 'C002',
      name: 'Priya Sharma',
      language: 'hi',
      segment: 'young_pro',
      current_provider: 'iifl',
      estimated_outstanding: 300000,
      estimated_rate: 21,
      city: 'Delhi',
    },
    {
      id: 'C003',
      name: 'Venkat Rao',
      language: 'te',
      segment: 'trust_seeker',
      current_provider: 'iifl',
      estimated_outstanding: 500000,
      estimated_rate: 20,
      city: 'Hyderabad',
    },
    {
      id: 'C004',
      name: 'Suresh Menon',
      language: 'ml',
      segment: 'high_value',
      current_provider: 'manappuram',
      estimated_outstanding: 1200000,
      estimated_rate: 19,
      city: 'Kochi',
    },
    {
      id: 'C005',
      name: 'Lakshmi Devi',
      language: 'ta',
      segment: 'shakti',
      current_provider: 'manappuram',
      estimated_outstanding: 200000,
      estimated_rate: 22,
      city: 'Chennai',
    },
    {
      id: 'C006',
      name: 'Ravi Kumar',
      language: 'kn',
      segment: 'trust_seeker',
      current_provider: 'muthoot',
      estimated_outstanding: 450000,
      estimated_rate: 19,
      city: 'Bangalore',
    },
  ];
}

/**
 * Format currency in INR
 */
export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 0,
  }).format(amount);
}

/**
 * Calculate potential savings for a customer
 * Kotak rate assumed at 10% vs competitor rate
 */
export function calculateSavings(outstanding: number, currentRate: number): number {
  const kotakRate = 10;
  const currentInterest = outstanding * (currentRate / 100);
  const kotakInterest = outstanding * (kotakRate / 100);
  return currentInterest - kotakInterest;
}
