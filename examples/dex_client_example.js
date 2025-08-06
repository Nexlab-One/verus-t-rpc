/**
 * Verus RPC Client for DEX Integration
 * 
 * This example shows how a DEX application would integrate with a public
 * Verus RPC service using JWT authentication.
 */

class VerusRPCClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
        this.token = null;
        this.tokenExpiry = null;
    }

    /**
     * Request a JWT token for RPC access (Anonymous)
     * @param {string[]} permissions - Array of permissions (e.g., ['read', 'write'])
     * @returns {Promise<string>} JWT token
     */
    async requestToken(permissions = ['read', 'write']) {
        try {
            const response = await fetch(`${this.baseUrl}/token/issue`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    // No user_id needed for anonymous users
                    permissions: permissions,
                    client_ip: await this.getClientIP(),
                    user_agent: navigator.userAgent,
                }),
            });

            if (!response.ok) {
                const error = await response.text();
                throw new Error(`Token request failed: ${response.status} - ${error}`);
            }

            const data = await response.json();
            this.token = data.token;
            this.tokenExpiry = Date.now() + (data.expires_in * 1000);
            this.userId = data.user_id; // Store the generated anonymous user ID
            
            console.log(`Token issued successfully for anonymous user: ${data.user_id}`);
            console.log(`Token expires in ${data.expires_in} seconds.`);
            return data.token;
        } catch (error) {
            console.error('Failed to request token:', error);
            throw error;
        }
    }

    /**
     * Check if current token is valid and not expired
     * @returns {boolean}
     */
    isTokenValid() {
        return this.token && this.tokenExpiry && Date.now() < this.tokenExpiry;
    }

    /**
     * Make an RPC request with JWT authentication
     * @param {string} method - RPC method name
     * @param {Array} params - RPC parameters
     * @returns {Promise<Object>} RPC response
     */
    async makeRPCRequest(method, params = []) {
        // Check if we need a new token
        if (!this.isTokenValid()) {
            throw new Error('No valid token available. Please request a new token first.');
        }

        try {
            const response = await fetch(this.baseUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`,
                },
                body: JSON.stringify({
                    jsonrpc: '2.0',
                    method: method,
                    params: params,
                    id: Date.now(),
                }),
            });

            if (!response.ok) {
                const error = await response.text();
                throw new Error(`RPC request failed: ${response.status} - ${error}`);
            }

            const data = await response.json();
            
            if (data.error) {
                throw new Error(`RPC error: ${data.error.message}`);
            }

            return data.result;
        } catch (error) {
            console.error(`RPC request failed for method ${method}:`, error);
            throw error;
        }
    }

    /**
     * Get client IP address using a public service
     * @returns {Promise<string>}
     */
    async getClientIP() {
        try {
            const response = await fetch('https://api.ipify.org?format=json');
            const data = await response.json();
            return data.ip;
        } catch (error) {
            console.warn('Could not determine client IP, using fallback');
            return '127.0.0.1';
        }
    }

    /**
     * Validate current token
     * @returns {Promise<Object>} Validation result
     */
    async validateToken() {
        if (!this.token) {
            throw new Error('No token to validate');
        }

        try {
            const response = await fetch(`${this.baseUrl}/token/validate`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    token: this.token,
                    client_ip: await this.getClientIP(),
                }),
            });

            if (!response.ok) {
                throw new Error(`Token validation failed: ${response.status}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Token validation failed:', error);
            throw error;
        }
    }

    // Convenience methods for common DEX operations
    async getBalance(address) {
        return await this.makeRPCRequest('z_getbalance', [address]);
    }

    async getNewAddress() {
        return await this.makeRPCRequest('z_getnewaddress', []);
    }

    async getInfo() {
        return await this.makeRPCRequest('getinfo', []);
    }

    async getBlock(hash) {
        return await this.makeRPCRequest('getblock', [hash]);
    }

    async sendMany(fromAddress, toAddresses) {
        return await this.makeRPCRequest('z_sendmany', [fromAddress, toAddresses]);
    }
}

// Usage Example for DEX Integration
async function dexIntegrationExample() {
    console.log('🚀 Starting DEX Integration Example');
    
    // Initialize client
    const verusClient = new VerusRPCClient('https://your-verus-rpc.com');
    
    try {
        // Step 1: Request a token for the DEX user
        console.log('\n📝 Step 1: Requesting JWT token...');
        const token = await verusClient.requestToken(['read', 'write']);
        console.log('✅ Token received successfully');
        
        // Step 2: Validate the token
        console.log('\n🔍 Step 2: Validating token...');
        const validation = await verusClient.validateToken();
        console.log('✅ Token validation result:', validation);
        
        // Step 3: Get server info
        console.log('\n📊 Step 3: Getting server info...');
        const info = await verusClient.getInfo();
        console.log('✅ Server info:', info);
        
        // Step 4: Get a new address for the user
        console.log('\n🏠 Step 4: Getting new address...');
        const newAddress = await verusClient.getNewAddress();
        console.log('✅ New address:', newAddress);
        
        // Step 5: Check balance (if address exists)
        if (newAddress) {
            console.log('\n💰 Step 5: Checking balance...');
            const balance = await verusClient.getBalance(newAddress);
            console.log('✅ Balance:', balance);
        }
        
        console.log('\n🎉 DEX integration example completed successfully!');
        
    } catch (error) {
        console.error('❌ DEX integration failed:', error);
    }
}

// Error handling and retry logic
class VerusRPCClientWithRetry extends VerusRPCClient {
    constructor(baseUrl, maxRetries = 3) {
        super(baseUrl);
        this.maxRetries = maxRetries;
    }

    async makeRPCRequestWithRetry(method, params = [], retryCount = 0) {
        try {
            return await this.makeRPCRequest(method, params);
        } catch (error) {
            if (retryCount < this.maxRetries && this.shouldRetry(error)) {
                console.log(`Retrying RPC request (${retryCount + 1}/${this.maxRetries})...`);
                await this.delay(Math.pow(2, retryCount) * 1000); // Exponential backoff
                return await this.makeRPCRequestWithRetry(method, params, retryCount + 1);
            }
            throw error;
        }
    }

    shouldRetry(error) {
        // Retry on network errors or rate limiting
        return error.message.includes('rate limit') || 
               error.message.includes('network') ||
               error.message.includes('timeout');
    }

    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { VerusRPCClient, VerusRPCClientWithRetry };
}

// Run example if this file is executed directly
if (typeof window !== 'undefined') {
    // Browser environment
    window.VerusRPCClient = VerusRPCClient;
    window.VerusRPCClientWithRetry = VerusRPCClientWithRetry;
    
    // Uncomment to run example
    // dexIntegrationExample();
}
