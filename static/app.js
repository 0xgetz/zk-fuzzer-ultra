// ZK Circuit Fuzzer Dashboard JavaScript

// API base URL
const API_BASE = '/api';

// Update dashboard metrics
async function updateMetrics() {
    try {
        const response = await fetch(`${API_BASE}/metrics`);
        const data = await response.json();
        
        document.getElementById('active-campaigns').textContent = data.active_campaigns;
        document.getElementById('total-bugs').textContent = data.total_bugs_found;
        document.getElementById('inputs-tested').textContent = formatNumber(data.total_inputs_tested);
        document.getElementById('avg-coverage').textContent = data.average_coverage.toFixed(1) + '%';
    } catch (error) {
        console.error('Error fetching metrics:', error);
    }
}

// Load campaigns list
async function loadCampaigns() {
    try {
        const response = await fetch(`${API_BASE}/campaigns`);
        const data = await response.json();
        
        const campaignList = document.getElementById('campaign-list');
        campaignList.innerHTML = '';
        
        data.campaigns.forEach(campaign => {
            const campaignEl = document.createElement('div');
            campaignEl.className = 'campaign-item';
            campaignEl.innerHTML = `
                <div class="campaign-header">
                    <h3>${campaign.name}</h3>
                    <span class="campaign-status status-${campaign.status.toLowerCase()}">${campaign.status}</span>
                </div>
                <div class="campaign-details">
                    <p><strong>Target:</strong> ${campaign.target_type}</p>
                    <p><strong>Progress:</strong> ${campaign.progress.toFixed(1)}%</p>
                    <p><strong>Inputs:</strong> ${formatNumber(campaign.total_inputs)}</p>
                    <p><strong>Interesting:</strong> ${campaign.interesting_inputs}</p>
                    <p><strong>Crashes:</strong> ${campaign.crashes_found}</p>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${campaign.progress}%"></div>
                </div>
            `;
            campaignList.appendChild(campaignEl);
        });
    } catch (error) {
        console.error('Error fetching campaigns:', error);
    }
}

// Load bugs list
async function loadBugs() {
    try {
        const response = await fetch(`${API_BASE}/bugs`);
        const data = await response.json();
        
        const bugList = document.getElementById('bug-list');
        bugList.innerHTML = '';
        
        data.bugs.forEach(bug => {
            const bugEl = document.createElement('div');
            bugEl.className = 'bug-item';
            bugEl.innerHTML = `
                <div class="bug-header">
                    <h4>${bug.bug_type}</h4>
                    <span class="bug-severity severity-${bug.severity.toLowerCase()}">${bug.severity}</span>
                </div>
                <div class="bug-details">
                    <p><strong>Campaign:</strong> ${bug.campaign_id}</p>
                    <p><strong>Description:</strong> ${bug.description}</p>
                    <p><strong>Discovered:</strong> ${new Date(bug.timestamp).toLocaleString()}</p>
                </div>
            `;
            bugList.appendChild(bugEl);
        });
    } catch (error) {
        console.error('Error fetching bugs:', error);
    }
}

// Format large numbers with commas
function formatNumber(num) {
    return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

// Initialize dashboard
function initDashboard() {
    updateMetrics();
    loadCampaigns();
    loadBugs();
    
    // Refresh data every 30 seconds
    setInterval(() => {
        updateMetrics();
        loadCampaigns();
        loadBugs();
    }, 30000);
}

// Start the dashboard when DOM is ready
document.addEventListener('DOMContentLoaded', initDashboard);
