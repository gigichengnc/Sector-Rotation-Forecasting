// RRG Pro - AI Enhanced Trading Terminal
console.log('RRG Pro initializing...');

class RRGProApp {
    constructor() {
        this.currentView = 'dashboard';
        this.charts = {};
        this.data = {};
        this.isConnected = false;
        this.updateInterval = null;
        
        this.init();
    }

    init() {
        this.setupSplashScreen();
    }

    setupSplashScreen() {
        const splash = document.getElementById('splash-screen');
        const status = document.getElementById('splash-status');
        
        const steps = [
            'Initializing application...',
            'Loading market data...',
            'Connecting to data feeds...',
            'Starting AI engine...',
            'Ready to trade!'
        ];
        
        let step = 0;
        const interval = setInterval(() => {
            if (step < steps.length) {
                status.textContent = steps[step];
                step++;
            } else {
                clearInterval(interval);
                setTimeout(() => {
                    splash.style.opacity = '0';
                    setTimeout(() => {
                        splash.style.display = 'none';
                        document.getElementById('main-app').style.display = 'flex';
                        this.startApplication();
                    }, 500);
                }, 1000);
            }
        }, 800);
    }

    startApplication() {
        this.initializeCharts();
        this.loadMarketData();
        this.startDataUpdates();
        this.updateConnectionStatus(true);
        this.bindEvents();
    }

    initializeCharts() {
        this.initRRGChart();
        this.updateMarketOverview();
        this.updateSectorPerformance();
        this.updateWatchlist();
        this.updateAIInsights();
        this.updateNews();
    }

    initRRGChart() {
        const ctx = document.getElementById('rrg-chart')?.getContext('2d');
        if (!ctx) return;

        const data = this.generateRRGData();
        
        if (typeof Chart !== 'undefined') {
            this.charts.rrg = new Chart(ctx, {
                type: 'scatter',
                data: {
                    datasets: [{
                        label: 'Sectors',
                        data: data,
                        backgroundColor: data.map(d => d.color),
                        borderColor: data.map(d => d.color),
                        pointRadius: 10,
                        pointHoverRadius: 15
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        x: {
                            type: 'linear',
                            title: { display: true, text: 'RS-Ratio' },
                            min: 0.8, max: 1.2
                        },
                        y: {
                            title: { display: true, text: 'RS-Momentum' },
                            min: -0.1, max: 0.1
                        }
                    },
                    plugins: { legend: { display: false } }
                }
            });
        }
    }

    generateRRGData() {
        const sectors = [
            { symbol: 'XLK', name: 'Technology', color: '#00d4ff' },
            { symbol: 'XLF', name: 'Financial', color: '#00ff88' },
            { symbol: 'XLE', name: 'Energy', color: '#ff6b35' },
            { symbol: 'XLV', name: 'Healthcare', color: '#a855f7' },
            { symbol: 'XLI', name: 'Industrial', color: '#06b6d4' }
        ];

        return sectors.map(sector => {
            const rsRatio = 0.85 + Math.random() * 0.35;
            const rsMomentum = -0.08 + Math.random() * 0.16;
            
            return {
                x: rsRatio,
                y: rsMomentum,
                label: sector.symbol,
                name: sector.name,
                color: sector.color,
                quadrant: this.getQuadrant(rsRatio, rsMomentum)
            };
        });
    }

    getQuadrant(rsRatio, rsMomentum) {
        if (rsRatio >= 1 && rsMomentum >= 0) return 'Leading';
        if (rsRatio >= 1 && rsMomentum < 0) return 'Weakening';
        if (rsRatio < 1 && rsMomentum < 0) return 'Lagging';
        return 'Improving';
    }

    updateMarketOverview() {
        const container = document.getElementById('market-indices');
        if (!container) return;

        const indices = [
            { symbol: 'SPY', name: 'S&P 500', price: 425.67, change: 2.34 },
            { symbol: 'QQQ', name: 'NASDAQ', price: 378.42, change: 4.12 },
            { symbol: 'IWM', name: 'Russell 2000', price: 198.23, change: -1.45 }
        ];

        container.innerHTML = indices.map(index => 
            '<div class="index-item">' +
            '<div class="index-symbol">' + index.symbol + '</div>' +
            '<div class="index-name">' + index.name + '</div>' +
            '<div class="index-price">$' + index.price.toFixed(2) + '</div>' +
            '<div class="index-change ' + (index.change >= 0 ? 'positive' : 'negative') + '">' +
            (index.change >= 0 ? '+' : '') + index.change.toFixed(2) +
            '</div></div>'
        ).join('');
    }

    updateSectorPerformance() {
        const container = document.getElementById('performance-list');
        if (!container) return;

        const data = this.generateRRGData();
        const sorted = data.sort((a, b) => b.y - a.y);

        container.innerHTML = sorted.map((sector, index) => 
            '<div class="performance-item">' +
            '<div class="performance-rank">' + (index + 1) + '</div>' +
            '<div class="performance-symbol" style="color: ' + sector.color + '">' + sector.label + '</div>' +
            '<div class="performance-name">' + sector.name + '</div>' +
            '<div class="performance-quadrant">' + sector.quadrant + '</div>' +
            '</div>'
        ).join('');
    }

    updateWatchlist() {
        const container = document.getElementById('watchlist-table');
        if (!container) return;

        const watchlist = [
            { symbol: 'AAPL', name: 'Apple Inc.', price: 175.43, change: 2.34 },
            { symbol: 'MSFT', name: 'Microsoft Corp.', price: 378.85, change: -1.23 },
            { symbol: 'GOOGL', name: 'Alphabet Inc.', price: 142.56, change: 0.87 }
        ];

        let html = '<div class="watchlist-header"><div>Symbol</div><div>Price</div><div>Change</div></div>';
        
        watchlist.forEach(item => {
            html += '<div class="watchlist-row">' +
                '<div class="watchlist-symbol">' + item.symbol + '</div>' +
                '<div class="watchlist-price">$' + item.price.toFixed(2) + '</div>' +
                '<div class="watchlist-change ' + (item.change >= 0 ? 'positive' : 'negative') + '">' +
                (item.change >= 0 ? '+' : '') + item.change.toFixed(2) + '</div>' +
                '</div>';
        });
        
        container.innerHTML = html;
    }

    updateAIInsights() {
        const container = document.getElementById('insights-list');
        if (!container) return;

        const insights = [
            { type: 'bullish', text: 'Technology sector showing strong momentum', confidence: 85 },
            { type: 'neutral', text: 'Energy sector rotation detected', confidence: 72 },
            { type: 'bearish', text: 'Utilities showing weakness', confidence: 68 }
        ];

        container.innerHTML = insights.map(insight => 
            '<div class="insight-item ' + insight.type + '">' +
            '<div class="insight-content">' +
            '<div class="insight-text">' + insight.text + '</div>' +
            '<div class="insight-confidence">Confidence: ' + insight.confidence + '%</div>' +
            '</div></div>'
        ).join('');
    }

    updateNews() {
        const container = document.getElementById('news-list');
        if (!container) return;

        const news = [
            { title: 'Fed Signals Potential Rate Cut', time: '2 min ago' },
            { title: 'Tech Earnings Beat Expectations', time: '15 min ago' },
            { title: 'Energy Sector Rotation Continues', time: '32 min ago' }
        ];

        container.innerHTML = news.map(item => 
            '<div class="news-item">' +
            '<div class="news-title">' + item.title + '</div>' +
            '<div class="news-time">' + item.time + '</div>' +
            '</div>'
        ).join('');
    }

    loadMarketData() {
        this.data.rrg = this.generateRRGData();
        this.updateStatusBar();
    }

    startDataUpdates() {
        this.updateInterval = setInterval(() => {
            this.data.rrg = this.generateRRGData();
            
            if (this.charts.rrg) {
                this.charts.rrg.data.datasets[0].data = this.data.rrg;
                this.charts.rrg.update('none');
            }
            
            this.updateSectorPerformance();
            this.updateStatusBar();
        }, 10000);
    }

    updateConnectionStatus(connected) {
        this.isConnected = connected;
        const dot = document.getElementById('connection-dot');
        const text = document.getElementById('connection-text');
        
        if (dot && text) {
            dot.className = 'status-dot ' + (connected ? 'connected' : 'disconnected');
            text.textContent = connected ? 'Connected' : 'Disconnected';
        }
    }

    updateStatusBar() {
        const dataPoints = document.getElementById('data-points');
        const lastUpdate = document.getElementById('last-update');
        
        if (dataPoints) dataPoints.textContent = (Math.floor(Math.random() * 1000) + 5000).toLocaleString();
        if (lastUpdate) lastUpdate.textContent = new Date().toLocaleTimeString();
    }

    switchView(viewName) {
        document.querySelectorAll('.view-container').forEach(view => {
            view.classList.remove('active');
        });
        
        const targetView = document.getElementById(viewName + '-view');
        if (targetView) {
            targetView.classList.add('active');
        }
        
        document.querySelectorAll('.nav-tab').forEach(tab => {
            tab.classList.remove('active');
        });
        
        const activeTab = document.querySelector('[data-view="' + viewName + '"]');
        if (activeTab) {
            activeTab.classList.add('active');
        }
        
        this.currentView = viewName;
    }

    bindEvents() {
        // Navigation tabs
        document.querySelectorAll('.nav-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                const view = tab.getAttribute('data-view');
                this.switchView(view);
            });
        });

        // Refresh button
        const refreshBtn = document.getElementById('refresh-all');
        if (refreshBtn) {
            refreshBtn.addEventListener('click', () => {
                console.log('Refreshing data...');
                this.loadMarketData();
                this.initializeCharts();
                this.showNotification('Data refreshed successfully!');
            });
        }

        // Chart controls
        const playBtn = document.getElementById('play-animation');
        const pauseBtn = document.getElementById('pause-animation');
        const resetBtn = document.getElementById('reset-animation');
        
        if (playBtn) {
            playBtn.addEventListener('click', () => {
                console.log('Starting animation...');
                this.startAnimation();
            });
        }
        
        if (pauseBtn) {
            pauseBtn.addEventListener('click', () => {
                console.log('Pausing animation...');
                this.pauseAnimation();
            });
        }
        
        if (resetBtn) {
            resetBtn.addEventListener('click', () => {
                console.log('Resetting animation...');
                this.resetAnimation();
            });
        }

        // Timeframe selector
        const timeframeSelect = document.getElementById('timeframe-select');
        if (timeframeSelect) {
            timeframeSelect.addEventListener('change', (e) => {
                console.log('Timeframe changed to:', e.target.value);
                this.changeTimeframe(e.target.value);
            });
        }

        // Analysis tools
        document.querySelectorAll('.tool-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                console.log('Analysis tool selected:', btn.getAttribute('data-tool'));
            });
        });

        // Run analysis button
        const runAnalysisBtn = document.getElementById('run-analysis');
        if (runAnalysisBtn) {
            runAnalysisBtn.addEventListener('click', () => {
                console.log('Running analysis...');
                this.runAnalysis();
            });
        }

        // Widget buttons
        document.querySelectorAll('.widget-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                console.log('Widget button clicked:', btn.id);
                this.handleWidgetAction(btn.id);
            });
        });
    }

    startAnimation() {
        if (this.animationInterval) {
            clearInterval(this.animationInterval);
        }
        
        this.animationInterval = setInterval(() => {
            this.data.rrg = this.generateRRGData();
            if (this.charts.rrg) {
                this.charts.rrg.data.datasets[0].data = this.data.rrg;
                this.charts.rrg.update('active');
            }
            this.updateSectorPerformance();
        }, 2000);
        
        this.showNotification('Animation started');
    }

    pauseAnimation() {
        if (this.animationInterval) {
            clearInterval(this.animationInterval);
            this.animationInterval = null;
        }
        this.showNotification('Animation paused');
    }

    resetAnimation() {
        this.pauseAnimation();
        this.loadMarketData();
        this.initializeCharts();
        this.showNotification('Animation reset');
    }

    changeTimeframe(timeframe) {
        this.currentTimeframe = timeframe;
        this.loadMarketData();
        this.initializeCharts();
        this.showNotification(`Timeframe changed to ${timeframe}`);
    }

    runAnalysis() {
        const benchmark = document.getElementById('analysis-benchmark')?.value || 'SPY';
        const period = document.getElementById('analysis-period')?.value || '60';
        
        const resultsContainer = document.getElementById('analysis-results');
        if (resultsContainer) {
            resultsContainer.innerHTML = `
                <div class="analysis-loading">
                    <div class="loading-spinner"></div>
                    <p>Running analysis with ${benchmark} benchmark over ${period} days...</p>
                </div>
            `;
            
            setTimeout(() => {
                resultsContainer.innerHTML = `
                    <div class="analysis-complete">
                        <h3>Analysis Complete</h3>
                        <p>Benchmark: ${benchmark}</p>
                        <p>Period: ${period} days</p>
                        <p>Sectors analyzed: ${this.data.rrg?.length || 5}</p>
                        <div class="analysis-chart-placeholder">
                            <p>📊 Analysis charts would appear here</p>
                        </div>
                    </div>
                `;
            }, 2000);
        }
        
        this.showNotification('Analysis started...');
    }

    handleWidgetAction(buttonId) {
        switch(buttonId) {
            case 'sort-performance':
                this.sortSectorsByPerformance();
                break;
            case 'sort-momentum':
                this.sortSectorsByMomentum();
                break;
            case 'add-symbol':
                this.showAddSymbolModal();
                break;
            case 'edit-watchlist':
                this.editWatchlist();
                break;
            default:
                console.log('Widget action:', buttonId);
        }
    }

    sortSectorsByPerformance() {
        console.log('Sorting by performance...');
        this.updateSectorPerformance();
        this.showNotification('Sorted by performance');
    }

    sortSectorsByMomentum() {
        console.log('Sorting by momentum...');
        const container = document.getElementById('performance-list');
        if (container && this.data.rrg) {
            const sorted = [...this.data.rrg].sort((a, b) => Math.abs(b.y) - Math.abs(a.y));
            container.innerHTML = sorted.map((sector, index) => 
                '<div class="performance-item">' +
                '<div class="performance-rank">' + (index + 1) + '</div>' +
                '<div class="performance-symbol" style="color: ' + sector.color + '">' + sector.label + '</div>' +
                '<div class="performance-name">' + sector.name + '</div>' +
                '<div class="performance-quadrant">' + sector.quadrant + '</div>' +
                '</div>'
            ).join('');
        }
        this.showNotification('Sorted by momentum');
    }

    showAddSymbolModal() {
        console.log('Opening add symbol modal...');
        this.showNotification('Add symbol feature coming soon!');
    }

    editWatchlist() {
        console.log('Opening watchlist editor...');
        this.showNotification('Watchlist editor coming soon!');
    }

    showNotification(message) {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = 'notification';
        notification.textContent = message;
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: var(--accent-color);
            color: var(--text-inverse);
            padding: 12px 20px;
            border-radius: 6px;
            font-size: 0.9rem;
            font-weight: 500;
            z-index: 3000;
            box-shadow: var(--shadow-md);
            animation: slideInRight 0.3s ease-out;
        `;
        
        document.body.appendChild(notification);
        
        // Remove after 3 seconds
        setTimeout(() => {
            notification.style.animation = 'slideOutRight 0.3s ease-in';
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.parentNode.removeChild(notification);
                }
            }, 300);
        }, 3000);
    }
}

document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM loaded, starting RRG Pro...');
    window.rrgPro = new RRGProApp();
});

console.log('RRG Pro script loaded successfully');